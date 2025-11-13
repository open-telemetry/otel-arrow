// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module tests batching.rs logic.

use crate::encode::{encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch};
use crate::otap::OtapArrowRecords;
use crate::otap::batching::make_output_batches;
use crate::otlp::OtlpProtoBytes;
use crate::payload::OtapPayload;
use crate::proto::OtlpProtoMessage;
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
use crate::testing::equiv::assert_equivalent;
use prost::Message as ProstMessage;
use std::num::NonZeroU64;

fn encode_otlp(msg: &OtlpProtoMessage) -> OtapArrowRecords {
    match msg {
        OtlpProtoMessage::Logs(logs) => encode_logs_otap_batch(logs),
        OtlpProtoMessage::Traces(traces) => encode_spans_otap_batch(traces),
        OtlpProtoMessage::Metrics(metrics) => encode_metrics_otap_batch(metrics),
    }
    .expect("encode ok")
}

fn decode_otlp(otap: OtapArrowRecords) -> OtlpProtoMessage {
    let pdata: OtapPayload = otap.into();
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

/// Create test OTLP logs data, 3 log records
fn make_logs_otlp(timestamp_offset: u64) -> OtlpProtoMessage {
    OtlpProtoMessage::Logs(LogsData::new(vec![
        ResourceLogs::new(
            Resource::build().finish(),
            vec![
                ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("scope".to_string())
                        .finish(),
                    vec![
                        LogRecord::build()
                            .time_unix_nano(1000u64 + timestamp_offset)
                            .observed_time_unix_nano(1100u64 + timestamp_offset)
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
                            .time_unix_nano(2000u64 + timestamp_offset)
                            .observed_time_unix_nano(2100u64 + timestamp_offset)
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
                        .time_unix_nano(3000u64 + timestamp_offset)
                        .observed_time_unix_nano(3100u64 + timestamp_offset)
                        .severity_number(SeverityNumber::Info as i32)
                        .finish(),
                ],
            )],
        ),
    ]))
}

/// Create test OTLP traces data with 3 simple spans
fn make_traces_otlp(timestamp_offset: u64) -> OtlpProtoMessage {
    OtlpProtoMessage::Traces(TracesData::new(vec![ResourceSpans::new(
        Resource::build().finish(),
        vec![ScopeSpans::new(
            InstrumentationScope::build().finish(),
            vec![
                Span::build()
                    .trace_id(vec![0u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span0".to_string())
                    .start_time_unix_nano(1000u64 + timestamp_offset)
                    .end_time_unix_nano(2000u64 + timestamp_offset)
                    .status(Status::new(StatusCode::Ok, "ok"))
                    .finish(),
                Span::build()
                    .trace_id(vec![0u8; 16])
                    .span_id(vec![2u8; 8])
                    .name("span1".to_string())
                    .start_time_unix_nano(3000u64 + timestamp_offset)
                    .end_time_unix_nano(4000u64 + timestamp_offset)
                    .status(Status::new(StatusCode::Ok, "ok"))
                    .finish(),
                Span::build()
                    .trace_id(vec![0u8; 16])
                    .span_id(vec![3u8; 8])
                    .name("span2".to_string())
                    .start_time_unix_nano(5000u64 + timestamp_offset)
                    .end_time_unix_nano(6000u64 + timestamp_offset)
                    .status(Status::new(StatusCode::Ok, "ok"))
                    .finish(),
            ],
        )],
    )]))
}

/// Create test OTLP metrics data
fn make_metrics_otlp(timestamp_offset: u64) -> OtlpProtoMessage {
    OtlpProtoMessage::Metrics(MetricsData::new(vec![ResourceMetrics::new(
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
                                time_unix_nano: 1000 + timestamp_offset,
                                value: Some(Value::AsDouble(10.0)),
                                ..Default::default()
                            },
                            NumberDataPoint {
                                time_unix_nano: 2000 + timestamp_offset,
                                value: Some(Value::AsDouble(20.0)),
                                ..Default::default()
                            },
                            NumberDataPoint {
                                time_unix_nano: 3000 + timestamp_offset,
                                value: Some(Value::AsDouble(1200.0)),
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
                                time_unix_nano: 3000 + timestamp_offset,
                                value: Some(Value::AsDouble(30.0)),
                                ..Default::default()
                            },
                            NumberDataPoint {
                                time_unix_nano: 4000 + timestamp_offset,
                                value: Some(Value::AsDouble(40.0)),
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
                            time_unix_nano: 5000 + timestamp_offset,
                            value: Some(Value::AsDouble(50.0)),
                            ..Default::default()
                        }],
                        aggregation_temporality: 1,
                        is_monotonic: true,
                    })),
                },
            ],
        )],
    )]))
}

/// Generic test function for batching across all signal types.
fn test_batching(inputs_otlp: Vec<OtlpProtoMessage>, max_output_batch: Option<NonZeroU64>) {
    let signal_type = inputs_otlp
        .first()
        .expect("at least one input")
        .signal_type();

    let inputs_otap: Vec<_> = inputs_otlp.iter().map(encode_otlp).collect();

    let outputs_otlp: Vec<_> = make_output_batches(signal_type, inputs_otap, max_output_batch)
        .expect("batching should succeed")
        .into_iter()
        .map(decode_otlp)
        .collect();

    // Assert batch_length <= max_output_batch
    if let Some(max_batch) = max_output_batch {
        for (i, output) in outputs_otlp.iter().enumerate() {
            let batch_len = output.batch_length();
            assert!(
                batch_len <= max_batch.get() as usize,
                "batch {} length {} exceeds limit {}",
                i,
                batch_len,
                max_batch
            );
        }
    }

    // Check OTLP equivalence
    assert_equivalent(&inputs_otlp, &outputs_otlp);
}

// Note: the tests below are quite simple, all they do is repeat
// similar data N times, but with a very fixed subset of the OTLP
// model: no attributes, no scope, only repetition with timestam
// variation.
//
// We envision extending this to more synthetic and corner-case data
// in a future PR.

#[test]
fn test_simple_batch_logs() {
    for input_count in 1..=20 {
        for max_output_batch in 3..=5 {
            let inputs: Vec<_> = (0..input_count)
                .map(|i| make_logs_otlp(i * 10000))
                .collect();
            test_batching(inputs, Some(NonZeroU64::new(max_output_batch).unwrap()));
        }
    }
}

#[test]
fn test_simple_batch_traces() {
    for input_count in 1..=20 {
        for max_output_batch in 3..=5 {
            let inputs: Vec<_> = (0..input_count)
                .map(|i| make_traces_otlp(i * 10000))
                .collect();
            test_batching(inputs, Some(NonZeroU64::new(max_output_batch).unwrap()));
        }
    }
}

#[test]
fn test_simple_batch_metrics() {
    for input_count in 1..=20 {
        for max_output_batch in 3..=5 {
            let inputs: Vec<_> = (0..input_count)
                .map(|i| make_metrics_otlp(i * 10000))
                .collect();
            test_batching(inputs, Some(NonZeroU64::new(max_output_batch).unwrap()));
        }
    }
}
