// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Payload decode unit tests: OTLP-proto, OTAP-proto, and Syslog decoders per signal, plus poison-input handling.

use super::*;

// ---- Routing and payload correctness ----

/// Scenario (routing and payload correctness): OTLP-proto traces bytes are decoded.
/// Guarantees: the payload decodes to an `ExportTracesRequest`, so OTLP-proto traces
/// route to the traces decoder.
#[test]
fn decode_traces_payload_otlp_proto() {
    let req = create_traces_with_spans();
    let mut bytes = vec![];
    req.encode(&mut bytes).expect("encode");

    let mut pdata = decode_traces_payload(&bytes, MessageFormat::OtlpProto).expect("should decode");
    let proto: OtlpProtoBytes = pdata
        .take_payload()
        .try_into_with_default()
        .expect("to OtlpProtoBytes");
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
        decode_metrics_payload(&bytes, MessageFormat::OtlpProto).expect("should decode");
    let proto: OtlpProtoBytes = pdata
        .take_payload()
        .try_into_with_default()
        .expect("to OtlpProtoBytes");
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

    let mut pdata = decode_logs_payload(&bytes, MessageFormat::OtlpProto).expect("should decode");
    let proto: OtlpProtoBytes = pdata
        .take_payload()
        .try_into_with_default()
        .expect("to OtlpProtoBytes");
    assert!(matches!(proto, OtlpProtoBytes::ExportLogsRequest(_)));
}

/// Scenario (routing and payload correctness): OTAP-Arrow traces bytes are decoded.
/// Guarantees: the payload decodes to `OtapArrowRecords::Traces`, so OTAP-encoded
/// traces route to the Arrow decoder.
#[test]
fn decode_traces_payload_otap_proto() {
    let bytes = create_traces_with_spans_otap_bytes();

    let mut pdata = decode_traces_payload(&bytes, MessageFormat::OtapProto).expect("should decode");
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
        decode_metrics_payload(&bytes, MessageFormat::OtapProto).expect("should decode");
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

    let mut pdata = decode_logs_payload(&bytes, MessageFormat::OtapProto).expect("should decode");
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
    let mut pdata = decode_logs_payload(input, MessageFormat::Syslog).expect("decode Syslog");
    let proto: OtlpProtoBytes = pdata
        .take_payload()
        .try_into_with_default()
        .expect("convert Syslog Arrow logs to OTLP");
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
    let mut pdata = decode_logs_payload(input, MessageFormat::Syslog).expect("decode Syslog CEF");
    let proto: OtlpProtoBytes = pdata
        .take_payload()
        .try_into_with_default()
        .expect("convert Syslog Arrow logs to OTLP");
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
    let result = decode_logs_payload(b"", MessageFormat::Syslog);
    assert!(result.is_err());
}

/// Scenario (routing and payload correctness): a traces or metrics message-format
/// header requests Syslog.
/// Guarantees: runtime header overrides cannot route Syslog bytes into non-log signals.
#[test]
fn decode_non_logs_payload_syslog_returns_error() {
    assert!(decode_traces_payload(b"message", MessageFormat::Syslog).is_err());
    assert!(decode_metrics_payload(b"message", MessageFormat::Syslog).is_err());
}

/// Scenario (routing and payload correctness): undecodable bytes are passed to the OTAP
/// traces decoder.
/// Guarantees: decode returns an error rather than panicking, so a malformed OTAP
/// payload is a recoverable per-message error.
#[test]
fn decode_traces_payload_invalid_otap_bytes_returns_error() {
    let result = decode_traces_payload(b"not valid protobuf", MessageFormat::OtapProto);
    assert!(result.is_err());
}

/// Scenario (routing and payload correctness): OTLP-proto traces bytes are decoded and
/// then re-extracted.
/// Guarantees: the bytes round-trip byte-for-byte, so the zero-copy OTLP path does not
/// mutate the payload.
#[test]
fn decode_traces_payload_otlp_preserves_bytes() {
    let req = create_traces_with_spans();
    let mut bytes = vec![];
    req.encode(&mut bytes).expect("encode");

    let mut pdata = decode_traces_payload(&bytes, MessageFormat::OtlpProto).expect("decode");
    let proto: OtlpProtoBytes = pdata
        .take_payload()
        .try_into_with_default()
        .expect("convert");
    assert_eq!(proto.as_bytes(), &bytes);
}
