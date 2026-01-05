// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Round-trip testing utilities for OTLP <-> OTAP conversions.
//!
//! This module provides encoding, decoding, and round-trip testing helpers
//! for all three telemetry signal types (logs, traces, metrics).

use crate::encode::{encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch};
use crate::otap::OtapArrowRecords;
use crate::otlp::OtlpProtoBytes;
use crate::payload::OtapPayload;
use crate::proto::OtlpProtoMessage;
use crate::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
use crate::proto::opentelemetry::metrics::v1::{
    Metric, MetricsData, ResourceMetrics, ScopeMetrics,
};
use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData};
use crate::testing::equiv::assert_equivalent;
use prost::Message as ProstMessage;

/// Transcode a protocol message object to OTAP records.
#[must_use]
pub fn otlp_to_otap(msg: &OtlpProtoMessage) -> OtapArrowRecords {
    match msg {
        OtlpProtoMessage::Logs(logs) => encode_logs(logs),
        OtlpProtoMessage::Metrics(metrics) => encode_metrics(metrics),
        OtlpProtoMessage::Traces(traces) => encode_traces(traces),
    }
}

/// Transcode OTAP records into a protocol message object.
#[must_use]
pub fn otap_to_otlp(otap: &OtapArrowRecords) -> OtlpProtoMessage {
    let pdata: OtapPayload = otap.clone().into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportLogsRequest(bytes) => {
            LogsData::decode(bytes.as_ref()).map(OtlpProtoMessage::Logs)
        }
        OtlpProtoBytes::ExportTracesRequest(bytes) => {
            TracesData::decode(bytes.as_ref()).map(OtlpProtoMessage::Traces)
        }
        OtlpProtoBytes::ExportMetricsRequest(bytes) => {
            MetricsData::decode(bytes.as_ref()).map(OtlpProtoMessage::Metrics)
        }
    }
    .expect("decode ok")
}

//
// Logs
//

/// Encode OTLP LogsData to OTAP Arrow Records
#[must_use]
pub fn encode_logs(logs: &LogsData) -> OtapArrowRecords {
    encode_logs_otap_batch(logs).expect("encode should succeed")
}

/// Decode OTAP Arrow Records to OTLP LogsData
#[must_use]
pub fn decode_logs(otap: OtapArrowRecords) -> LogsData {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportLogsRequest(bytes) => {
            LogsData::decode(bytes.as_ref()).expect("decode should succeed")
        }
        _ => panic!("expected logs"),
    }
}

/// Round-trip test: OTLP -> OTAP -> OTLP and verify equivalence
pub fn test_logs_round_trip(input: LogsData) {
    let encoded = encode_logs(&input);
    let decoded = decode_logs(encoded);

    assert_equivalent(&[input.into()], &[decoded.into()]);
}

/// helper function for converting [`LogRecord`]s to [`LogsData`]
#[must_use]
pub fn to_logs_data(log_records: Vec<LogRecord>) -> LogsData {
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

/// helper function for converting OTLP logs to OTAP batch
#[must_use]
pub fn to_otap_logs(log_records: Vec<LogRecord>) -> OtapArrowRecords {
    otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records)))
}

//
// Traces
//

/// Encode OTLP TracesData to OTAP Arrow Records
#[must_use]
pub fn encode_traces(traces: &TracesData) -> OtapArrowRecords {
    encode_spans_otap_batch(traces).expect("encode should succeed")
}

/// Decode OTAP Arrow Records to OTLP TracesData
#[must_use]
pub fn decode_traces(otap: OtapArrowRecords) -> TracesData {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportTracesRequest(bytes) => {
            TracesData::decode(bytes.as_ref()).expect("decode should succeed")
        }
        _ => panic!("expected traces"),
    }
}

/// Round-trip test: OTLP -> OTAP -> OTLP and verify equivalence
pub fn test_traces_round_trip(input: TracesData) {
    let encoded = encode_traces(&input);
    let decoded = decode_traces(encoded);

    let input_msg: OtlpProtoMessage = input.into();
    let decoded_msg: OtlpProtoMessage = decoded.into();

    assert_equivalent(&[input_msg], &[decoded_msg]);
}

/// helper function for converting [`Span`]s to [`TracesData`]
#[must_use]
pub fn to_traces_data(spans: Vec<Span>) -> TracesData {
    TracesData {
        resource_spans: vec![ResourceSpans {
            scope_spans: vec![ScopeSpans {
                spans,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

/// helper function for converting OTLP spans to OTAP batch
#[must_use]
pub fn to_otap_traces(spans: Vec<Span>) -> OtapArrowRecords {
    otlp_to_otap(&OtlpProtoMessage::Traces(to_traces_data(spans)))
}

//
// Metrics
//

/// Encode OTLP MetricsData to OTAP Arrow Records
#[must_use]
pub fn encode_metrics(metrics: &MetricsData) -> OtapArrowRecords {
    encode_metrics_otap_batch(metrics).expect("encode should succeed")
}

/// Decode OTAP Arrow Records to OTLP MetricsData
#[must_use]
pub fn decode_metrics(otap: OtapArrowRecords) -> MetricsData {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportMetricsRequest(bytes) => {
            MetricsData::decode(bytes.as_ref()).expect("decode should succeed")
        }
        _ => panic!("expected metrics"),
    }
}

/// Round-trip test: OTLP -> OTAP -> OTLP and verify equivalence
pub fn test_metrics_round_trip(input: MetricsData) {
    let encoded = encode_metrics(&input);
    let decoded = decode_metrics(encoded);

    let input_msg: OtlpProtoMessage = input.into();
    let decoded_msg: OtlpProtoMessage = decoded.into();

    assert_equivalent(&[input_msg], &[decoded_msg]);
}

/// helper function for converting [`Metric`]s to [`MetricsData`]
#[must_use]
pub fn to_metrics_data(metrics: Vec<Metric>) -> MetricsData {
    MetricsData {
        resource_metrics: vec![ResourceMetrics {
            scope_metrics: vec![ScopeMetrics {
                metrics,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

/// helper function for converting OTLP metrics to OTAP batch
#[must_use]
pub fn to_otap_metrics(metrics: Vec<Metric>) -> OtapArrowRecords {
    otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)))
}
