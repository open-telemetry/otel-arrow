// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for common.rs optional resource and scope handling.
//!
//! This module tests the round-trip encoding/decoding of OTLP data
//! with optional resource, scope, and attributes fields. The changes
//! in common.rs make ResourceArrays::id and ScopeArrays fields optional,
//! allowing OTAP batches to omit these columns when not present.

#[cfg(test)]
mod tests {
    use crate::encode::{encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch};
    use crate::otap::OtapArrowRecords;
    use crate::otlp::OtlpProtoBytes;
    use crate::payload::OtapPayload;
    use crate::proto::OtlpProtoMessage;
    use crate::proto::opentelemetry::common::v1::InstrumentationScope;
    use crate::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use crate::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
        ScopeMetrics, Sum,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, TracesData, span::SpanKind,
    };
    use crate::testing::equiv::assert_equivalent;
    use prost::Message as ProstMessage;

    /// Encode OTLP LogsData to OTAP Arrow Records
    fn encode_logs(logs: &LogsData) -> OtapArrowRecords {
        encode_logs_otap_batch(logs).expect("encode should succeed")
    }

    /// Decode OTAP Arrow Records back to OTLP LogsData
    fn decode_logs(otap: OtapArrowRecords) -> LogsData {
        let pdata: OtapPayload = otap.into();
        let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
        match otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                LogsData::decode(bytes.as_ref()).expect("decode should succeed")
            }
            _ => panic!("expected logs"),
        }
    }

    /// Perform round-trip test: OTLP -> OTAP -> OTLP and verify equivalence
    fn test_logs_round_trip(input: LogsData) {
        let encoded = encode_logs(&input);
        let decoded = decode_logs(encoded);

        let input_msg = OtlpProtoMessage::Logs(input);
        let decoded_msg = OtlpProtoMessage::Logs(decoded);

        assert_equivalent(&[input_msg], &[decoded_msg]);
    }

    /// Encode OTLP TracesData to OTAP Arrow Records
    fn encode_traces(traces: &TracesData) -> OtapArrowRecords {
        encode_spans_otap_batch(traces).expect("encode should succeed")
    }

    /// Decode OTAP Arrow Records back to OTLP TracesData
    fn decode_traces(otap: OtapArrowRecords) -> TracesData {
        let pdata: OtapPayload = otap.into();
        let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
        match otlp_bytes {
            OtlpProtoBytes::ExportTracesRequest(bytes) => {
                TracesData::decode(bytes.as_ref()).expect("decode should succeed")
            }
            _ => panic!("expected traces"),
        }
    }

    /// Perform round-trip test: OTLP -> OTAP -> OTLP and verify equivalence
    fn test_traces_round_trip(input: TracesData) {
        let encoded = encode_traces(&input);
        let decoded = decode_traces(encoded);

        let input_msg = OtlpProtoMessage::Traces(input);
        let decoded_msg = OtlpProtoMessage::Traces(decoded);

        assert_equivalent(&[input_msg], &[decoded_msg]);
    }

    #[test]
    fn test_logs_with_full_resource_and_scope() {
        // Test the normal case: logs with resource and scope
        let logs = LogsData::new(vec![ResourceLogs::new(
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
        )]);

        test_logs_round_trip(logs);
    }

    #[test]
    fn test_logs_with_no_resource() {
        // Test case 1: Logs with no resource
        // When resource is None, the OTAP encoding should omit the resource column
        let logs = LogsData::new(vec![ResourceLogs::new(
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
        )]);

        test_logs_round_trip(logs);
    }

    #[test]
    fn test_logs_with_no_scope() {
        // Test case 2: Logs with resource but no scope
        // When scope has no meaningful data, it should still round-trip correctly
        let logs = LogsData::new(vec![ResourceLogs::new(
            Resource::build().finish(),
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
        )]);

        test_logs_round_trip(logs);
    }

    #[test]
    fn test_logs_with_no_attributes() {
        // Test case 3: Logs with resource and scope but no attributes
        // This tests that attributes columns can be optional
        let logs = LogsData::new(vec![ResourceLogs::new(
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
        )]);

        test_logs_round_trip(logs);
    }

    #[test]
    fn test_logs_with_no_resource_no_scope() {
        // Test case 4: Logs with neither resource nor scope data
        // This is the minimal case
        let logs = LogsData::new(vec![ResourceLogs::new(
            Resource::default(),
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
        )]);

        test_logs_round_trip(logs);
    }

    #[test]
    fn test_logs_multiple_records_no_resource() {
        // Test multiple log records with no resource
        let logs = LogsData::new(vec![ResourceLogs::new(
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
        )]);

        test_logs_round_trip(logs);
    }

    #[test]
    fn test_logs_multiple_scopes_no_resource() {
        // Test multiple scopes with no resource
        let logs = LogsData::new(vec![ResourceLogs::new(
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
        )]);

        test_logs_round_trip(logs);
    }

    #[test]
    fn test_logs_multiple_resources_mixed_content() {
        // Test multiple resources with different content
        let logs = LogsData::new(vec![
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
        ]);

        test_logs_round_trip(logs);
    }

    #[test]
    fn test_empty_logs() {
        // Edge case: completely empty logs data
        // Per OpenTelemetry spec, empty envelopes can be dropped
        // We return an empty OtapArrowRecords with batch_length() == 0
        let logs = LogsData::new(vec![]);
        let encoded = encode_logs(&logs);
        assert_eq!(
            encoded.batch_length(),
            0,
            "Empty logs should have batch_length == 0"
        );
    }

    #[test]
    fn test_logs_with_empty_scope_logs() {
        // Edge case: resource with no scope logs (no actual log records)
        // This should succeed and return empty batch
        let logs = LogsData::new(vec![ResourceLogs::new(
            Resource::build().finish(),
            vec![], // No scope logs
        )]);

        let encoded = encode_logs(&logs);
        assert_eq!(
            encoded.batch_length(),
            0,
            "Logs with no records should have batch_length == 0"
        );
    }

    #[test]
    fn test_logs_with_empty_log_records() {
        // Edge case: scope with no log records
        let logs = LogsData::new(vec![ResourceLogs::new(
            Resource::build().finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope".to_string())
                    .finish(),
                vec![], // No log records
            )],
        )]);

        test_logs_round_trip(logs);
    }

    // ============================================================================
    // Traces Tests
    // ============================================================================

    #[test]
    fn test_traces_with_full_resource_and_scope() {
        // Test the normal case: traces with resource and scope
        let traces = TracesData::new(vec![ResourceSpans::new(
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
        )]);

        test_traces_round_trip(traces);
    }

    #[test]
    fn test_traces_with_no_resource() {
        // Test case 1: Traces with no resource
        // When resource is empty, the OTAP encoding should handle it correctly
        let traces = TracesData::new(vec![ResourceSpans::new(
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
        )]);

        test_traces_round_trip(traces);
    }

    #[test]
    fn test_traces_with_no_scope() {
        // Test case 2: Traces with resource but no scope
        // When scope has no meaningful data, it should still round-trip correctly
        let traces = TracesData::new(vec![ResourceSpans::new(
            Resource::build().finish(),
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
        )]);

        test_traces_round_trip(traces);
    }

    #[test]
    fn test_traces_with_no_attributes() {
        // Test case 3: Traces with resource and scope but no attributes
        // This tests that attributes columns can be optional
        let traces = TracesData::new(vec![ResourceSpans::new(
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
        )]);

        test_traces_round_trip(traces);
    }

    #[test]
    fn test_traces_with_no_resource_no_scope() {
        // Test case 4: Traces with neither resource nor scope data
        // This is the minimal case
        let traces = TracesData::new(vec![ResourceSpans::new(
            Resource::default(),
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
        )]);

        test_traces_round_trip(traces);
    }

    #[test]
    fn test_traces_multiple_spans_no_resource() {
        // Test multiple spans with no resource
        let traces = TracesData::new(vec![ResourceSpans::new(
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
        )]);

        test_traces_round_trip(traces);
    }

    #[test]
    fn test_traces_multiple_scopes_no_resource() {
        // Test multiple scopes with no resource
        let traces = TracesData::new(vec![ResourceSpans::new(
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
        )]);

        test_traces_round_trip(traces);
    }

    #[test]
    fn test_traces_multiple_resources_mixed_content() {
        // Test multiple resources with different content
        let traces = TracesData::new(vec![
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
        ]);

        test_traces_round_trip(traces);
    }

    #[test]
    fn test_empty_traces() {
        // Edge case: completely empty traces data
        // Per OpenTelemetry spec, empty envelopes can be dropped
        // See: https://github.com/open-telemetry/opentelemetry-proto/issues/598
        // We return an empty OtapArrowRecords with batch_length() == 0
        let traces = TracesData::new(vec![]);
        let encoded = encode_traces(&traces);
        assert_eq!(
            encoded.batch_length(),
            0,
            "Empty traces should have batch_length == 0"
        );
    }

    #[test]
    fn test_traces_with_empty_scope_spans() {
        // Edge case: resource with no scope spans (no actual spans)
        // This should succeed and return empty batch
        let traces = TracesData::new(vec![ResourceSpans::new(
            Resource::build().finish(),
            vec![], // No scope spans
        )]);

        let encoded = encode_traces(&traces);
        assert_eq!(
            encoded.batch_length(),
            0,
            "Traces with no spans should have batch_length == 0"
        );
    }

    #[test]
    fn test_traces_with_empty_spans() {
        // Edge case: scope with no spans
        let traces = TracesData::new(vec![ResourceSpans::new(
            Resource::build().finish(),
            vec![ScopeSpans::new(
                InstrumentationScope::build()
                    .name("scope".to_string())
                    .finish(),
                vec![], // No spans
            )],
        )]);

        let encoded = encode_traces(&traces);
        assert_eq!(
            encoded.batch_length(),
            0,
            "Traces with no spans should have batch_length == 0"
        );
    }

    // ============================================================================
    // Metrics Tests (Sum only)
    // ============================================================================

    /// Encode OTLP MetricsData to OTAP Arrow Records
    fn encode_metrics(metrics: &MetricsData) -> OtapArrowRecords {
        encode_metrics_otap_batch(metrics).expect("encode should succeed")
    }

    /// Decode OTAP Arrow Records back to OTLP MetricsData
    fn decode_metrics(otap: OtapArrowRecords) -> MetricsData {
        let pdata: OtapPayload = otap.into();
        let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
        match otlp_bytes {
            OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                MetricsData::decode(bytes.as_ref()).expect("decode should succeed")
            }
            _ => panic!("expected metrics"),
        }
    }

    /// Perform round-trip test: OTLP -> OTAP -> OTLP and verify equivalence
    fn test_metrics_round_trip(input: MetricsData) {
        let encoded = encode_metrics(&input);
        let decoded = decode_metrics(encoded);

        let input_msg = OtlpProtoMessage::Metrics(input);
        let decoded_msg = OtlpProtoMessage::Metrics(decoded);

        assert_equivalent(&[input_msg], &[decoded_msg]);
    }

    #[test]
    fn test_metrics_sum_with_full_resource_and_scope() {
        // Test the normal case: metrics with resource, scope, and data points
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build()
                    .name("test-scope".to_string())
                    .finish(),
                vec![Metric::build()
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
                                .value_int(84i64)
                                .finish(),
                        ],
                    ))
                    .finish()],
            )],
        )]);

        test_metrics_round_trip(metrics);
    }

    #[test]
    fn test_metrics_sum_with_no_resource() {
        // Test case 1: Metrics with no resource
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(), // Empty resource
            vec![ScopeMetrics::new(
                InstrumentationScope::build()
                    .name("test-scope".to_string())
                    .finish(),
                vec![Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![NumberDataPoint::build()
                            .time_unix_nano(1000u64)
                            .value_int(42i64)
                            .finish()],
                    ))
                    .finish()],
            )],
        )]);

        test_metrics_round_trip(metrics);
    }

    #[test]
    fn test_metrics_sum_with_no_scope() {
        // Test case 2: Metrics with resource but no scope
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(), // Empty scope
                vec![Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![NumberDataPoint::build()
                            .time_unix_nano(1000u64)
                            .value_int(42i64)
                            .finish()],
                    ))
                    .finish()],
            )],
        )]);

        test_metrics_round_trip(metrics);
    }

    #[test]
    fn test_metrics_sum_with_no_resource_no_scope() {
        // Test case 3: Metrics with neither resource nor scope
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::default(),
                vec![Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![NumberDataPoint::build()
                            .time_unix_nano(1000u64)
                            .value_int(42i64)
                            .finish()],
                    ))
                    .finish()],
            )],
        )]);

        test_metrics_round_trip(metrics);
    }

    #[test]
    fn test_metrics_sum_with_no_data_points() {
        // Test case 4: Sum with no data points
        // This is a key test - a metric with no points should be handled gracefully
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build()
                    .name("test-scope".to_string())
                    .finish(),
                vec![Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![], // No data points
                    ))
                    .finish()],
            )],
        )]);

        test_metrics_round_trip(metrics);
    }

    #[test]
    fn test_empty_metrics() {
        // Edge case: completely empty metrics data
        // Per OpenTelemetry spec, empty envelopes can be dropped
        // See: https://github.com/open-telemetry/opentelemetry-proto/issues/598
        let metrics = MetricsData::new(vec![]);
        let encoded = encode_metrics(&metrics);
        assert_eq!(
            encoded.batch_length(),
            0,
            "Empty metrics should have batch_length == 0"
        );
    }

    #[test]
    fn test_metrics_with_no_scope_metrics() {
        // Edge case: resource with no scope metrics (no actual metrics)
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![], // No scope metrics
        )]);

        let encoded = encode_metrics(&metrics);
        assert_eq!(
            encoded.batch_length(),
            0,
            "Metrics with no scope metrics should have batch_length == 0"
        );
    }

    #[test]
    fn test_metrics_with_no_metrics() {
        // Edge case: scope with no metrics
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build()
                    .name("scope".to_string())
                    .finish(),
                vec![], // No metrics
            )],
        )]);

        let encoded = encode_metrics(&metrics);
        assert_eq!(
            encoded.batch_length(),
            0,
            "Metrics with no metrics should have batch_length == 0"
        );
    }

    #[test]
    fn test_metrics_multiple_sums_no_resource() {
        // Test multiple Sum metrics with no resource
        let metrics = MetricsData::new(vec![ResourceMetrics::new(
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
                            vec![NumberDataPoint::build()
                                .time_unix_nano(1000u64)
                                .value_int(10i64)
                                .finish()],
                        ))
                        .finish(),
                    Metric::build()
                        .name("test.counter2")
                        .data_sum(Sum::new(
                            AggregationTemporality::Delta,
                            false,
                            vec![NumberDataPoint::build()
                                .time_unix_nano(2000u64)
                                .value_double(3.14)
                                .finish()],
                        ))
                        .finish(),
                ],
            )],
        )]);

        test_metrics_round_trip(metrics);
    }
}
