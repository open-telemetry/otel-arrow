// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Payload decode unit tests: OTLP-proto, OTLP-JSON, OTAP-proto, and Syslog decoders per signal, plus poison-input handling.

use super::*;

// ---- Routing and payload correctness ----

/// Scenario (routing and payload correctness): OTLP-proto traces bytes are decoded.
/// Guarantees: the payload decodes to an `ExportTracesRequest`, so OTLP-proto traces
/// route to the traces decoder.
#[test]
fn decode_traces_payload_otlp_proto() {
    let bytes = encoded_trace_fixture();

    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Traces, &bytes, MessageFormat::OtlpProto)
            .expect("should decode");
    let proto = take_otlp_proto(&mut pdata);
    assert!(matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)));
}

/// Scenario (routing and payload correctness): OTLP-proto metrics bytes are decoded.
/// Guarantees: the payload decodes to an `ExportMetricsRequest`, so OTLP-proto metrics
/// route to the metrics decoder.
#[test]
fn decode_metrics_payload_otlp_proto() {
    let req = create_metrics_service_request();
    let mut bytes = vec![];
    req.encode(&mut bytes).expect("encode");

    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Metrics, &bytes, MessageFormat::OtlpProto)
            .expect("should decode");
    let proto = take_otlp_proto(&mut pdata);
    assert!(matches!(proto, OtlpProtoBytes::ExportMetricsRequest(_)));
}

/// Scenario (routing and payload correctness): OTLP-proto logs bytes are decoded.
/// Guarantees: the payload decodes to an `ExportLogsRequest`, so OTLP-proto logs route
/// to the logs decoder.
#[test]
fn decode_logs_payload_otlp_proto() {
    let req = create_logs_service_request();
    let mut bytes = vec![];
    req.encode(&mut bytes).expect("encode");

    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Logs, &bytes, MessageFormat::OtlpProto)
            .expect("should decode");
    let proto = take_otlp_proto(&mut pdata);
    assert!(matches!(proto, OtlpProtoBytes::ExportLogsRequest(_)));
}

/// Scenario (routing and payload correctness): valid OTLP JSON requests are decoded
/// for traces, metrics, and logs.
/// Guarantees: each JSON document becomes the matching OTLP protobuf request variant
/// while preserving representative signal data.
#[test]
fn decode_otlp_json_payloads() {
    let cases = [
        (
            SignalType::Traces,
            br#"{"resourceSpans":[{"scopeSpans":[{"spans":[{"traceId":"01010101010101010101010101010101","spanId":"0202020202020202","name":"operation"}]}]}]}"#
                .as_slice(),
        ),
        (
            SignalType::Metrics,
            br#"{"resourceMetrics":[{"scopeMetrics":[{"metrics":[{"name":"requests","gauge":{"dataPoints":[{"timeUnixNano":"42","asInt":"7"}]}}]}]}]}"#
                .as_slice(),
        ),
        (
            SignalType::Logs,
            br#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[{"timeUnixNano":"42","severityNumber":9,"body":{"stringValue":"ready"}}]}]}]}"#
                .as_slice(),
        ),
    ];

    for (signal, json) in cases {
        let mut pdata = SignalDecoder::decode_signal_payload(signal, json, MessageFormat::OtlpJson)
            .expect("valid OTLP JSON should decode");
        let proto = take_otlp_proto(&mut pdata);

        match (signal, proto) {
            (SignalType::Traces, OtlpProtoBytes::ExportTracesRequest(bytes)) => {
                let request =
                    ExportTraceServiceRequest::decode(bytes).expect("decode traces protobuf");
                assert_eq!(
                    request.resource_spans[0].scope_spans[0].spans[0].name,
                    "operation"
                );
            }
            (SignalType::Metrics, OtlpProtoBytes::ExportMetricsRequest(bytes)) => {
                let request =
                    ExportMetricsServiceRequest::decode(bytes).expect("decode metrics protobuf");
                let metric = &request.resource_metrics[0].scope_metrics[0].metrics[0];
                assert_eq!(metric.name, "requests");
                let gauge = match metric.data.as_ref() {
                    Some(otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data::Gauge(
                        gauge,
                    )) => gauge,
                    _ => panic!("metric should decode as a gauge"),
                };
                assert!(matches!(
                    gauge.data_points[0].value,
                    Some(
                        otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::number_data_point::Value::AsInt(
                            7
                        )
                    )
                ));
            }
            (SignalType::Logs, OtlpProtoBytes::ExportLogsRequest(bytes)) => {
                let request =
                    ExportLogsServiceRequest::decode(bytes).expect("decode logs protobuf");
                assert_eq!(
                    request.resource_logs[0].scope_logs[0].log_records[0].severity_number,
                    9
                );
            }
            _ => panic!("signal routed to the wrong OTLP request variant"),
        }
    }
}

/// Scenario (routing and payload correctness): OTLP JSON logs contain an empty
/// array body and an explicitly null optional enum field.
/// Guarantees: the empty array is preserved and the null severity is treated as
/// unset, matching ProtoJSON semantics.
#[test]
fn decode_otlp_json_accepts_empty_array_and_null_optional_field() {
    let json = br#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[{"severityNumber":null,"body":{"arrayValue":{}}}]}]}]}"#;

    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Logs, json, MessageFormat::OtlpJson)
            .expect("valid OTLP JSON defaults should decode");
    let proto = take_otlp_proto(&mut pdata);
    let request = ExportLogsServiceRequest::decode(proto.as_bytes()).expect("decode logs protobuf");
    let record = &request.resource_logs[0].scope_logs[0].log_records[0];

    assert_eq!(record.severity_number, 0);
    assert!(matches!(
        record.body.as_ref().and_then(|body| body.value.as_ref()),
        Some(any_value::Value::ArrayValue(array)) if array.values.is_empty()
    ));
}

/// Scenario (routing and payload correctness): malformed JSON is received with the
/// OTLP JSON message format.
/// Guarantees: decoding returns a recoverable pdata conversion error instead of
/// forwarding invalid bytes or exposing payload values in internal errors.
#[test]
fn decode_invalid_otlp_json_payload_returns_error() {
    const SENSITIVE_VALUE: &str = "customer-secret-value";
    let payload = format!(r#"{{"resourceLogs":"{SENSITIVE_VALUE}"}}"#);
    let error = SignalDecoder::decode_signal_payload(
        SignalType::Logs,
        payload.as_bytes(),
        MessageFormat::OtlpJson,
    )
    .expect_err("invalid OTLP JSON must fail");

    let message = error.to_string();
    assert!(message.contains("Failed to decode OTLP JSON logs payload: Data error at line"));
    assert!(!message.contains(SENSITIVE_VALUE));
}

/// Scenario (routing and payload correctness): OTAP-Arrow traces bytes are decoded.
/// Guarantees: the payload decodes to `OtapArrowRecords::Traces`, so OTAP-encoded
/// traces route to the Arrow decoder.
#[test]
fn decode_traces_payload_otap_proto() {
    let bytes = create_traces_with_spans_otap_bytes();

    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Traces, &bytes, MessageFormat::OtapProto)
            .expect("should decode");
    let payload: OtapPayload = pdata.take_payload();
    assert!(
        matches!(
            payload.into_data(),
            PayloadData::OtapArrowRecords(OtapArrowRecords::Traces(_))
        ),
        "expected OtapArrowRecords::Traces"
    );
}

/// Scenario (routing and payload correctness): OTAP-Arrow metrics bytes are decoded.
/// Guarantees: the payload decodes to `OtapArrowRecords::Metrics`, so OTAP-encoded
/// metrics route to the Arrow decoder.
#[test]
fn decode_metrics_payload_otap_proto() {
    let bytes = create_metrics_otap_arrow_records_bytes();

    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Metrics, &bytes, MessageFormat::OtapProto)
            .expect("should decode");
    let payload: OtapPayload = pdata.take_payload();
    assert!(
        matches!(
            payload.into_data(),
            PayloadData::OtapArrowRecords(OtapArrowRecords::Metrics(_))
        ),
        "expected OtapArrowRecords::Metrics"
    );
}

/// Scenario (routing and payload correctness): OTAP-Arrow logs bytes are decoded.
/// Guarantees: the payload decodes to `OtapArrowRecords::Logs`, so OTAP-encoded logs
/// route to the Arrow decoder.
#[test]
fn decode_logs_payload_otap_proto() {
    let bytes = create_logs_otap_arrow_records_bytes();

    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Logs, &bytes, MessageFormat::OtapProto)
            .expect("should decode");
    let payload: OtapPayload = pdata.take_payload();
    assert!(
        matches!(
            payload.into_data(),
            PayloadData::OtapArrowRecords(OtapArrowRecords::Logs(_))
        ),
        "expected OtapArrowRecords::Logs"
    );
}

/// Scenario (routing and payload correctness): one RFC 5424 message is received in a
/// Kafka record configured for Syslog.
/// Guarantees: the shared Syslog parser produces exactly one OpenTelemetry log record.
#[test]
fn decode_logs_payload_syslog_rfc5424() {
    let input = b"<34>1 2003-10-11T22:14:15.003Z host app - ID47 - Test message";
    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Logs, input, MessageFormat::Syslog)
            .expect("decode Syslog");
    let proto = take_otlp_proto(&mut pdata);
    let request = ExportLogsServiceRequest::decode(proto.as_bytes()).expect("decode OTLP logs");

    assert_eq!(request.resource_logs.len(), 1);
    assert_eq!(request.resource_logs[0].scope_logs.len(), 1);
    assert_eq!(request.resource_logs[0].scope_logs[0].log_records.len(), 1);
}

/// Scenario (routing and payload correctness): an RFC 3164 message contains a CEF body.
/// Guarantees: Kafka Syslog decoding reuses the existing CEF mapping and emits CEF
/// attributes on the OpenTelemetry log record.
#[test]
fn decode_logs_payload_syslog_with_embedded_cef() {
    let input = b"<34>Oct 11 22:14:15 firewall CEF:0|Vendor|Product|2.0|signature-123|Intrusion detected|7|act=blocked";
    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Logs, input, MessageFormat::Syslog)
            .expect("decode Syslog CEF");
    let proto = take_otlp_proto(&mut pdata);
    let request = ExportLogsServiceRequest::decode(proto.as_bytes()).expect("decode OTLP logs");
    let attributes = &request.resource_logs[0].scope_logs[0].log_records[0].attributes;

    assert!(attributes.iter().any(|attribute| {
        attribute.key == "cef.device_vendor"
                && attribute
                    .value
                    .as_ref()
                    .and_then(|value| value.value.as_ref())
                    .is_some_and(|value| {
                        matches!(value, any_value::Value::StringValue(value) if value == "Vendor")
                    })
    }));
}

/// Scenario (routing and payload correctness): an empty Kafka record is decoded as
/// Syslog.
/// Guarantees: malformed Syslog returns a recoverable per-message error instead of
/// panicking or stalling the consumer loop.
#[test]
fn decode_logs_payload_empty_syslog_returns_error() {
    let result = SignalDecoder::decode_signal_payload(SignalType::Logs, b"", MessageFormat::Syslog);
    assert!(result.is_err());
}

/// Scenario (routing and payload correctness): a traces or metrics message-format
/// header requests Syslog.
/// Guarantees: runtime header overrides cannot route Syslog bytes into non-log signals.
#[test]
fn decode_non_logs_payload_syslog_returns_error() {
    assert!(
        SignalDecoder::decode_signal_payload(SignalType::Traces, b"message", MessageFormat::Syslog)
            .is_err()
    );
    assert!(
        SignalDecoder::decode_signal_payload(
            SignalType::Metrics,
            b"message",
            MessageFormat::Syslog
        )
        .is_err()
    );
}

/// Scenario (routing and payload correctness): undecodable bytes are passed to the OTAP
/// traces decoder.
/// Guarantees: decode returns an error rather than panicking, so a malformed OTAP
/// payload is a recoverable per-message error.
#[test]
fn decode_traces_payload_invalid_otap_bytes_returns_error() {
    let result = SignalDecoder::decode_signal_payload(
        SignalType::Traces,
        b"not valid protobuf",
        MessageFormat::OtapProto,
    );
    assert!(result.is_err());
}

/// Scenario (routing and payload correctness): OTLP-proto traces bytes are decoded and
/// then re-extracted.
/// Guarantees: the bytes round-trip byte-for-byte, so the zero-copy OTLP path does not
/// mutate the payload.
#[test]
fn decode_traces_payload_otlp_preserves_bytes() {
    let bytes = encoded_trace_fixture();

    let mut pdata =
        SignalDecoder::decode_signal_payload(SignalType::Traces, &bytes, MessageFormat::OtlpProto)
            .expect("decode");
    let proto = take_otlp_proto(&mut pdata);
    assert_eq!(proto.as_bytes(), &bytes);
}
