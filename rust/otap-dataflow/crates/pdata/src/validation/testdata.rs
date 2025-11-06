// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;

pub mod traces {
    use super::*;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, status::StatusCode,
    };
    use crate::{SpanID, TraceID};

    pub fn create_single_request() -> ExportTraceServiceRequest {
        let start_time = 1619712000000000000u64;
        let end_time = 1619712001000000000u64;
        let trace_id = TraceID::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let span_id = SpanID::new(&[1, 2, 3, 4, 5, 6, 7, 8]);

        let span = Span::build()
            .trace_id(trace_id)
            .span_id(span_id)
            .name("test_span")
            .start_time_unix_nano(start_time)
            .end_time_unix_nano(end_time)
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .status(Status::new(StatusCode::Ok, "success"))
            .finish();

        ExportTraceServiceRequest::new(vec![ResourceSpans::new(
            Resource::default(),
            vec![ScopeSpans::new(InstrumentationScope::default(), vec![span])],
        )])
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

        let data_point = NumberDataPoint::build()
            .start_time_unix_nano(timestamp)
            .time_unix_nano(timestamp + 1000000000)
            .value_double(42.0)
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .finish();

        let metric = Metric::build()
            .name("test_gauge")
            .data_gauge(Gauge::new(vec![data_point]))
            .description("Test metric".to_string())
            .unit("count")
            .finish();

        ExportMetricsServiceRequest::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![metric],
            )],
        )])
    }
}

pub mod logs {
    use super::*;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;

    const EVENT_NAME: &str = "event123";
    const TIMESTAMP: u64 = 1619712000000000000u64;

    pub fn to_export_logs_request(log_records: Vec<LogRecord>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource::default(),
            vec![ScopeLogs::new(
                InstrumentationScope::build().name("scope_name").finish(),
                log_records,
            )],
        )])
    }

    pub fn create_single_request() -> ExportLogsServiceRequest {
        let log_record = LogRecord::build()
            .time_unix_nano(TIMESTAMP)
            .severity_number(SeverityNumber::Info)
            .event_name(EVENT_NAME)
            .severity_text("INFO")
            .body(AnyValue::new_string("Test log message".to_string()))
            .attributes(vec![
                KeyValue::new("test.attribute", AnyValue::new_string("test value")),
                KeyValue::new(
                    "test.map.attribute",
                    AnyValue::new_kvlist(vec![KeyValue::new(
                        "attr_map_k1",
                        AnyValue::new_string("attr_map_v1"),
                    )]),
                ),
                KeyValue::new(
                    "test.list.attribute",
                    AnyValue::new_array((0..3).map(AnyValue::new_int).collect::<Vec<_>>()),
                ),
            ])
            .trace_id((0u8..16u8).collect::<Vec<u8>>())
            .span_id((0u8..8u8).collect::<Vec<u8>>())
            .finish();

        to_export_logs_request(vec![log_record])
    }

    /// Create requests where OTAP should serialize the log bodies using cbor
    pub fn create_request_with_serialized_bodies() -> ExportLogsServiceRequest {
        let list_body_log_record = LogRecord::build()
            .time_unix_nano(TIMESTAMP)
            .severity_number(SeverityNumber::Info)
            .event_name(EVENT_NAME)
            .severity_text("INFO")
            .body(AnyValue::new_array(vec![
                AnyValue::new_string("test1"),
                AnyValue::new_string("test2"),
            ]))
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .trace_id((0u8..16u8).collect::<Vec<u8>>())
            .span_id((0u8..8u8).collect::<Vec<u8>>())
            .finish();

        let map_body_log_record = LogRecord::build()
            .time_unix_nano(TIMESTAMP)
            .severity_number(SeverityNumber::Debug)
            .event_name(EVENT_NAME)
            .severity_text("DEBUG")
            .body(AnyValue::new_kvlist(vec![
                // test serialization/deserialization of all supported types ..
                KeyValue::new("test_map_str", AnyValue::new_string("str_val")),
                KeyValue::new("test_map_int", AnyValue::new_int(99)),
                KeyValue::new("test_map_f64", AnyValue::new_double(5.0)),
                KeyValue::new("test_map_bool", AnyValue::new_bool(true)),
                KeyValue::new("test_map_bytes", AnyValue::new_bytes(b"123")),
                KeyValue::new("test_map_nil", AnyValue { value: None }),
                KeyValue::new(
                    "test_map_nested_list",
                    AnyValue::new_array(vec![AnyValue::new_string("list_val_1")]),
                ),
                KeyValue::new(
                    "test_map_nested_map",
                    AnyValue::new_kvlist(vec![KeyValue::new(
                        "child_map_key_1",
                        AnyValue::new_string("child_str_val_1"),
                    )]),
                ),
            ]))
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value2"),
            )])
            .trace_id((8u8..24u8).collect::<Vec<u8>>())
            .span_id((8u8..16u8).collect::<Vec<u8>>())
            .finish();

        to_export_logs_request(vec![list_body_log_record, map_body_log_record])
    }
}
