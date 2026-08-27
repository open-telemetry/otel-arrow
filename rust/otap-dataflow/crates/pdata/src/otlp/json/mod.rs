// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP JSON encoding for backend-agnostic pdata views.
//!
//! The signal-specific writers traverse pdata view traits directly and stream one compact OTLP
//! JSON document to a caller-provided writer. This avoids materializing an intermediate protobuf
//! object while preserving OTLP rules for identifiers, enums, and 64-bit values. Callers remain
//! responsible for framing, output limits, and recovery from partial writes.

mod common;
mod logs;
mod metrics;
mod traces;

use serde::Serialize;
use std::io::Write;
use thiserror::Error;

pub use logs::write_logs_json;
pub use metrics::write_metrics_json;
pub use traces::write_traces_json;

/// Error returned while encoding one OTLP JSON document.
#[derive(Debug, Error)]
#[error("could not encode OTLP JSON: {0}")]
pub struct JsonEncodeError(#[from] serde_json::Error);

fn write_json<T: Serialize, W: Write>(value: &T, output: &mut W) -> Result<(), JsonEncodeError> {
    serde_json::to_writer(output, value).map_err(JsonEncodeError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{
        encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch,
    };
    use crate::proto::opentelemetry::common::v1::{
        AnyValue, ArrayValue, KeyValue, KeyValueList, any_value,
    };
    use crate::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
    use crate::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge,
        Histogram, HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
        ScopeMetrics, Sum, Summary, SummaryDataPoint, exponential_histogram_data_point, metric,
        number_data_point, summary_data_point,
    };
    use crate::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, TracesData, span,
    };
    use crate::views::otap::{OtapLogsView, OtapMetricsView, OtapTracesView};
    use crate::views::otlp::bytes::logs::RawLogsData;
    use crate::views::otlp::bytes::metrics::RawMetricsData;
    use crate::views::otlp::bytes::traces::RawTraceData;
    use prost::Message;
    use serde_json::json;
    use std::io::{self, Write};

    fn encoded<M: Message>(message: &M) -> Vec<u8> {
        let mut bytes = Vec::new();
        message.encode(&mut bytes).unwrap();
        bytes
    }

    fn decoded_document(document: &[u8]) -> serde_json::Value {
        serde_json::from_slice(document).unwrap()
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("injected writer failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn sample_logs() -> LogsData {
        LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Default::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(Default::default()),
                    log_records: vec![LogRecord {
                        time_unix_nano: 42,
                        severity_number: 9,
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("ready".to_owned())),
                        }),
                        trace_id: vec![1; 16],
                        span_id: vec![2; 8],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn sample_metrics() -> MetricsData {
        MetricsData {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Default::default()),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(Default::default()),
                    metrics: vec![Metric {
                        name: "requests".to_owned(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                start_time_unix_nano: 10,
                                time_unix_nano: 20,
                                value: Some(number_data_point::Value::AsInt(7)),
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn sample_traces() -> TracesData {
        TracesData {
            resource_spans: vec![ResourceSpans {
                resource: Some(Default::default()),
                scope_spans: vec![ScopeSpans {
                    scope: Some(Default::default()),
                    spans: vec![Span {
                        trace_id: vec![3; 16],
                        span_id: vec![4; 8],
                        name: "GET /ready".to_owned(),
                        kind: span::SpanKind::Server as i32,
                        start_time_unix_nano: 100,
                        end_time_unix_nano: 200,
                        status: Some(crate::proto::opentelemetry::trace::v1::Status {
                            code: 2,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    /// Scenario: An empty logs view is written to a reusable byte buffer.
    /// Guarantees: The serializer emits one compact JSON document without a framing delimiter.
    #[test]
    fn writes_one_unframed_compact_document() {
        let mut output = Vec::new();
        write_logs_json(&LogsData::default(), &mut output).unwrap();
        assert_eq!(output, b"{}");
    }

    /// Scenario: The caller-provided writer rejects an OTLP JSON write.
    /// Guarantees: The serializer returns a typed error that preserves the writer failure.
    #[test]
    fn reports_writer_failures() {
        let error = write_logs_json(&LogsData::default(), &mut FailingWriter).unwrap_err();
        assert!(error.to_string().contains("injected writer failure"));
    }

    /// Scenario: A protobuf logs batch contains timestamps, severity, a body, and trace context.
    /// Guarantees: Logs encode with OTLP field names, quoted 64-bit values, numeric enums, and hex IDs.
    #[test]
    fn logs_view_encodes_otlp_protojson_shape() {
        let request = sample_logs();
        let bytes = encoded(&request);
        let view = RawLogsData::try_new(&bytes).unwrap();
        let mut output = Vec::new();
        write_logs_json(&view, &mut output).unwrap();
        assert_eq!(
            decoded_document(&output),
            json!({
                "resourceLogs": [{
                    "resource": {},
                    "scopeLogs": [{
                        "scope": {},
                        "logRecords": [{
                            "timeUnixNano": "42",
                            "severityNumber": 9,
                            "body": {"stringValue": "ready"},
                            "traceId": "01010101010101010101010101010101",
                            "spanId": "0202020202020202"
                        }]
                    }]
                }]
            })
        );
    }

    /// Scenario: A protobuf gauge carries an integer data point and non-default timestamps.
    /// Guarantees: Metrics encode the data oneof and all protobuf 64-bit integers as JSON strings.
    #[test]
    fn metrics_view_encodes_otlp_protojson_shape() {
        let request = sample_metrics();
        let bytes = encoded(&request);
        let view = RawMetricsData::try_new(&bytes).unwrap();
        let mut output = Vec::new();
        write_metrics_json(&view, &mut output).unwrap();
        assert_eq!(
            decoded_document(&output),
            json!({
                "resourceMetrics": [{
                    "resource": {},
                    "scopeMetrics": [{
                        "scope": {},
                        "metrics": [{
                            "name": "requests",
                            "gauge": {"dataPoints": [{
                                "startTimeUnixNano": "10",
                                "timeUnixNano": "20",
                                "asInt": "7"
                            }]}
                        }]
                    }]
                }]
            })
        );
    }

    /// Scenario: A protobuf server span carries identity, timing, kind, and an error status.
    /// Guarantees: Traces encode hex IDs, quoted timestamps, and OTLP numeric enum values.
    #[test]
    fn traces_view_encodes_otlp_protojson_shape() {
        let request = sample_traces();
        let bytes = encoded(&request);
        let view = RawTraceData::try_new(&bytes).unwrap();
        let mut output = Vec::new();
        write_traces_json(&view, &mut output).unwrap();
        assert_eq!(
            decoded_document(&output),
            json!({
                "resourceSpans": [{
                    "resource": {},
                    "scopeSpans": [{
                        "scope": {},
                        "spans": [{
                            "traceId": "03030303030303030303030303030303",
                            "spanId": "0404040404040404",
                            "name": "GET /ready",
                            "kind": 2,
                            "startTimeUnixNano": "100",
                            "endTimeUnixNano": "200",
                            "status": {"code": 2}
                        }]
                    }]
                }]
            })
        );
    }

    /// Scenario: A log body contains nested arrays, key-value lists, integers, and bytes.
    /// Guarantees: Every AnyValue variant uses its OTLP JSON wrapper and special value encoding.
    #[test]
    fn nested_any_values_use_otlp_json_encoding() {
        let request = LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(any_value::Value::ArrayValue(ArrayValue {
                                values: vec![
                                    AnyValue {
                                        value: Some(any_value::Value::IntValue(-7)),
                                    },
                                    AnyValue {
                                        value: Some(any_value::Value::BytesValue(vec![0, 255])),
                                    },
                                    AnyValue {
                                        value: Some(any_value::Value::KvlistValue(KeyValueList {
                                            values: vec![KeyValue {
                                                key: "ready".to_owned(),
                                                value: Some(AnyValue {
                                                    value: Some(any_value::Value::BoolValue(true)),
                                                }),
                                            }],
                                        })),
                                    },
                                ],
                            })),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let mut output = Vec::new();
        write_logs_json(&request, &mut output).unwrap();
        assert_eq!(
            decoded_document(&output)["resourceLogs"][0]["scopeLogs"][0]["logRecords"][0]["body"],
            json!({"arrayValue": {"values": [
                {"intValue": "-7"},
                {"bytesValue": "AP8="},
                {"kvlistValue": {"values": [{
                    "key": "ready",
                    "value": {"boolValue": true}
                }]}}
            ]}})
        );
    }

    /// Scenario: Metrics contain gauge, sum, histogram, exponential histogram, and summary data.
    /// Guarantees: Every metric oneof is serialized and temporalities remain numeric OTLP enums.
    #[test]
    fn metrics_encode_every_data_variant() {
        let point = || NumberDataPoint {
            value: Some(number_data_point::Value::AsDouble(f64::INFINITY)),
            ..Default::default()
        };
        let request = MetricsData {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![
                        Metric {
                            name: "gauge".to_owned(),
                            data: Some(metric::Data::Gauge(Gauge {
                                data_points: vec![point()],
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "sum".to_owned(),
                            data: Some(metric::Data::Sum(Sum {
                                data_points: vec![point()],
                                aggregation_temporality: AggregationTemporality::Delta as i32,
                                is_monotonic: true,
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "histogram".to_owned(),
                            data: Some(metric::Data::Histogram(Histogram {
                                data_points: vec![HistogramDataPoint {
                                    count: 1,
                                    bucket_counts: vec![1],
                                    explicit_bounds: vec![1.5],
                                    ..Default::default()
                                }],
                                aggregation_temporality: AggregationTemporality::Cumulative as i32,
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "exponential".to_owned(),
                            data: Some(metric::Data::ExponentialHistogram(ExponentialHistogram {
                                data_points: vec![ExponentialHistogramDataPoint {
                                    count: 1,
                                    scale: -1,
                                    positive: Some(exponential_histogram_data_point::Buckets {
                                        offset: -2,
                                        bucket_counts: vec![1],
                                    }),
                                    ..Default::default()
                                }],
                                aggregation_temporality: AggregationTemporality::Delta as i32,
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "summary".to_owned(),
                            data: Some(metric::Data::Summary(Summary {
                                data_points: vec![SummaryDataPoint {
                                    count: 1,
                                    sum: 2.0,
                                    quantile_values: vec![summary_data_point::ValueAtQuantile {
                                        quantile: 0.5,
                                        value: 2.0,
                                    }],
                                    ..Default::default()
                                }],
                            })),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let mut output = Vec::new();
        write_metrics_json(&request, &mut output).unwrap();
        let document = decoded_document(&output);
        let metrics = document["resourceMetrics"][0]["scopeMetrics"][0]["metrics"]
            .as_array()
            .unwrap();
        assert_eq!(metrics.len(), 5);
        assert_eq!(metrics[0]["gauge"]["dataPoints"][0]["asDouble"], "Infinity");
        assert_eq!(metrics[1]["sum"]["aggregationTemporality"], 1);
        assert_eq!(metrics[2]["histogram"]["aggregationTemporality"], 2);
        assert!(metrics[3].get("exponentialHistogram").is_some());
        assert!(metrics[4].get("summary").is_some());
    }

    /// Scenario: Equivalent logs use owned protobuf, raw protobuf, and OTAP Arrow views.
    /// Guarantees: All supported logs backends produce semantically identical OTLP JSON.
    #[test]
    fn logs_encoding_matches_across_view_backends() {
        let request = sample_logs();
        let bytes = encoded(&request);
        let raw_view = RawLogsData::try_new(&bytes).unwrap();
        let records = encode_logs_otap_batch(&request).unwrap();
        let otap_view = OtapLogsView::try_from(&records).unwrap();
        let mut owned_output = Vec::new();
        let mut raw_output = Vec::new();
        let mut otap_output = Vec::new();
        write_logs_json(&request, &mut owned_output).unwrap();
        write_logs_json(&raw_view, &mut raw_output).unwrap();
        write_logs_json(&otap_view, &mut otap_output).unwrap();
        assert_eq!(
            decoded_document(&raw_output),
            decoded_document(&owned_output)
        );
        assert_eq!(
            decoded_document(&otap_output),
            decoded_document(&owned_output)
        );
    }

    /// Scenario: Equivalent metrics use owned protobuf, raw protobuf, and OTAP Arrow views.
    /// Guarantees: All supported metrics backends produce semantically identical OTLP JSON.
    #[test]
    fn metrics_encoding_matches_across_view_backends() {
        let request = sample_metrics();
        let bytes = encoded(&request);
        let raw_view = RawMetricsData::try_new(&bytes).unwrap();
        let records = encode_metrics_otap_batch(&request).unwrap();
        let otap_view = OtapMetricsView::try_from(&records).unwrap();
        let mut owned_output = Vec::new();
        let mut raw_output = Vec::new();
        let mut otap_output = Vec::new();
        write_metrics_json(&request, &mut owned_output).unwrap();
        write_metrics_json(&raw_view, &mut raw_output).unwrap();
        write_metrics_json(&otap_view, &mut otap_output).unwrap();
        assert_eq!(
            decoded_document(&raw_output),
            decoded_document(&owned_output)
        );
        assert_eq!(
            decoded_document(&otap_output),
            decoded_document(&owned_output)
        );
    }

    /// Scenario: Equivalent traces use owned protobuf, raw protobuf, and OTAP Arrow views.
    /// Guarantees: All supported traces backends produce semantically identical OTLP JSON.
    #[test]
    fn traces_encoding_matches_across_view_backends() {
        let request = sample_traces();
        let bytes = encoded(&request);
        let raw_view = RawTraceData::try_new(&bytes).unwrap();
        let records = encode_spans_otap_batch(&request).unwrap();
        let otap_view = OtapTracesView::try_from(&records).unwrap();
        let mut owned_output = Vec::new();
        let mut raw_output = Vec::new();
        let mut otap_output = Vec::new();
        write_traces_json(&request, &mut owned_output).unwrap();
        write_traces_json(&raw_view, &mut raw_output).unwrap();
        write_traces_json(&otap_view, &mut otap_output).unwrap();
        assert_eq!(
            decoded_document(&raw_output),
            decoded_document(&owned_output)
        );
        assert_eq!(
            decoded_document(&otap_output),
            decoded_document(&owned_output)
        );
    }

    /// Scenario: Every signal receives a top-level protobuf message with a truncated field body.
    /// Guarantees: Validating raw views reject malformed framing before any JSON bytes are emitted.
    #[test]
    fn raw_signal_views_reject_malformed_top_level_protobuf() {
        let malformed = [0x0a, 0x05, 0x01];
        assert!(RawLogsData::try_new(&malformed).is_err());
        assert!(RawMetricsData::try_new(&malformed).is_err());
        assert!(RawTraceData::try_new(&malformed).is_err());
    }
}
