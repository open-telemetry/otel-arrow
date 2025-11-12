// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module tests batching.rs logic.

use crate::encode::{encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch};
use crate::otap::OtapArrowRecords;
use crate::otap::batching::make_output_batches;
use crate::otlp::OtlpProtoBytes;
use crate::payload::OtapPayload;
use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use crate::proto::opentelemetry::metrics::v1::{
    Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, metric::Data,
    number_data_point::Value,
};
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, TracesData, status::StatusCode,
};
use crate::testing::equiv::assert_logs_equivalent;
use crate::testing::equiv::assert_metrics_equivalent;
use crate::testing::equiv::assert_traces_equivalent;
use otap_df_config::SignalType;
use pretty_assertions::assert_eq;
use prost::Message as ProstMessage;
use std::num::NonZeroU64;

fn encode_logs(logs: &LogsData) -> OtapArrowRecords {
    encode_logs_otap_batch(logs).expect("encode logs")
}

fn encode_traces(traces: &TracesData) -> OtapArrowRecords {
    encode_spans_otap_batch(traces).expect("encode traces")
}

fn encode_metrics(metrics: &MetricsData) -> OtapArrowRecords {
    encode_metrics_otap_batch(metrics).expect("encode metrics")
}

fn decode_logs(otap: OtapArrowRecords) -> LogsData {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportLogsRequest(bytes) => {
            LogsData::decode(bytes.as_ref()).expect("decode logs")
        }
        _ => panic!("expected logs"),
    }
}

fn decode_traces(otap: OtapArrowRecords) -> TracesData {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportTracesRequest(bytes) => {
            TracesData::decode(bytes.as_ref()).expect("decode traces")
        }
        _ => panic!("expected traces"),
    }
}

fn decode_metrics(otap: OtapArrowRecords) -> MetricsData {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportMetricsRequest(bytes) => {
            MetricsData::decode(bytes.as_ref()).expect("decode metrics")
        }
        _ => panic!("expected metrics"),
    }
}

/// Create test OTLP logs data with 3 log records across 2 resources and 3 scopes
fn make_logs_otlp() -> LogsData {
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
        ),
        ResourceLogs::new(
            Resource::build().finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope".to_string())
                    .finish(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(3000u64)
                        .observed_time_unix_nano(3100u64)
                        .severity_number(SeverityNumber::Info as i32)
                        .finish(),
                ],
            )],
        ),
    ])
}

/// Create test OTLP traces data with 4 simple spans
fn make_traces_otlp() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::build().finish(),
        vec![ScopeSpans::new(
            InstrumentationScope::build().finish(),
            vec![
                // Span 0
                Span::build()
                    .trace_id(vec![0u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span0".to_string())
                    .start_time_unix_nano(1000u64)
                    .end_time_unix_nano(2000u64)
                    .status(Status::new(StatusCode::Ok, "ok"))
                    .finish(),
                // Span 1
                Span::build()
                    .trace_id(vec![0u8; 16])
                    .span_id(vec![2u8; 8])
                    .name("span1".to_string())
                    .start_time_unix_nano(3000u64)
                    .end_time_unix_nano(4000u64)
                    .status(Status::new(StatusCode::Ok, "ok"))
                    .finish(),
                // Span 2
                Span::build()
                    .trace_id(vec![0u8; 16])
                    .span_id(vec![3u8; 8])
                    .name("span2".to_string())
                    .start_time_unix_nano(5000u64)
                    .end_time_unix_nano(6000u64)
                    .status(Status::new(StatusCode::Ok, "ok"))
                    .finish(),
                // Span 3
                Span::build()
                    .trace_id(vec![0u8; 16])
                    .span_id(vec![4u8; 8])
                    .name("span3".to_string())
                    .start_time_unix_nano(7000u64)
                    .end_time_unix_nano(8000u64)
                    .status(Status::new(StatusCode::Ok, "ok"))
                    .finish(),
            ],
        )],
    )])
}

/// Create test OTLP metrics data with 3 metrics (2 Gauges with 2 data points each, 1 Sum with 1 data point)
fn make_metrics_otlp() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build().finish(),
            vec![
                // Gauge metric with 2 data points
                Metric {
                    name: "gauge1".into(),
                    description: "First gauge".into(),
                    unit: "1".into(),
                    metadata: vec![],
                    data: Some(Data::Gauge(Gauge {
                        data_points: vec![
                            NumberDataPoint {
                                time_unix_nano: 1000,
                                value: Some(Value::AsDouble(10.0)),
                                ..Default::default()
                            },
                            NumberDataPoint {
                                time_unix_nano: 2000,
                                value: Some(Value::AsDouble(20.0)),
                                ..Default::default()
                            },
                        ],
                    })),
                },
                // Gauge metric with 2 data points
                Metric {
                    name: "gauge2".into(),
                    description: "Second gauge".into(),
                    unit: "1".into(),
                    metadata: vec![],
                    data: Some(Data::Gauge(Gauge {
                        data_points: vec![
                            NumberDataPoint {
                                time_unix_nano: 3000,
                                value: Some(Value::AsDouble(30.0)),
                                ..Default::default()
                            },
                            NumberDataPoint {
                                time_unix_nano: 4000,
                                value: Some(Value::AsDouble(40.0)),
                                ..Default::default()
                            },
                        ],
                    })),
                },
                // Sum metric with 1 data point
                Metric {
                    name: "sum1".into(),
                    description: "A sum".into(),
                    unit: "1".into(),
                    metadata: vec![],
                    data: Some(Data::Sum(Sum {
                        data_points: vec![NumberDataPoint {
                            time_unix_nano: 5000,
                            value: Some(Value::AsDouble(50.0)),
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

#[test]
fn test_simple_split_logs() {
    let logs_otlp = make_logs_otlp();
    let logs_otap = encode_logs(&logs_otlp);

    // Split with max_output_batch=2
    let result = make_output_batches(
        SignalType::Logs,
        vec![logs_otap],
        Some(NonZeroU64::new(2).unwrap()),
    )
    .expect("batching should succeed");

    // Merge back and verify equivalence
    let merged_result =
        make_output_batches(SignalType::Logs, result, None).expect("merge should succeed");

    let merged_otlp = decode_logs(merged_result[0].clone());
    assert_logs_equivalent(&logs_otlp, &merged_otlp);
}

#[test]
fn test_simple_split_traces() {
    let traces_otlp = make_traces_otlp();
    let traces_otap = encode_traces(&traces_otlp);

    // Split with max_output_batch=2
    let result = make_output_batches(
        SignalType::Traces,
        vec![traces_otap],
        Some(NonZeroU64::new(2).unwrap()),
    )
    .expect("batching should succeed");

    // Merge back and verify equivalence
    let merged_result =
        make_output_batches(SignalType::Traces, result, None).expect("merge should succeed");

    let merged_otlp = decode_traces(merged_result[0].clone());
    assert_traces_equivalent(&traces_otlp, &merged_otlp);
}

#[test]
fn test_simple_split_metrics() {
    let metrics_otlp = make_metrics_otlp();
    let metrics_otap = encode_metrics(&metrics_otlp);

    // Split with max_output_batch=2
    let result = make_output_batches(
        SignalType::Metrics,
        vec![metrics_otap],
        Some(NonZeroU64::new(2).unwrap()),
    )
    .expect("batching should succeed");

    // Merge back and verify equivalence
    let merged_result =
        make_output_batches(SignalType::Metrics, result, None).expect("merge should succeed");

    let merged_otlp = decode_metrics(merged_result[0].clone());
    assert_metrics_equivalent(&metrics_otlp, &merged_otlp);
}
