// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module tests batching.rs logic.

use crate::otlp::OtlpProtoBytes;
use crate::payload::OtapPayload;
use crate::proto::opentelemetry::collector::{
    logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
    trace::v1::ExportTraceServiceRequest,
};
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
use crate::{
    otap::{OtapArrowRecords, Traces},
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
};
use otap_df_config::SignalType;
use pretty_assertions::assert_eq;
use prost::Message as ProstMessage;
use std::num::NonZeroU64;

/// Helper to encode OTLP TracesData to OTAP
fn encode_traces(traces: &TracesData) -> OtapArrowRecords {
    crate::encode::encode_spans_otap_batch(traces).expect("encode traces")
}

/// Helper to encode OTLP LogsData to OTAP
fn encode_logs(logs: &LogsData) -> OtapArrowRecords {
    crate::encode::encode_logs_otap_batch(logs).expect("encode logs")
}

/// Helper to encode OTLP MetricsData to OTAP
fn encode_metrics(metrics: &MetricsData) -> OtapArrowRecords {
    crate::encode::encode_metrics_otap_batch(metrics).expect("encode metrics")
}

fn otap_to_otlp_traces(otap: OtapArrowRecords) -> ExportTraceServiceRequest {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportTracesRequest(bytes) => {
            ExportTraceServiceRequest::decode(bytes.as_ref()).expect("decode traces")
        }
        _ => panic!("expected traces"),
    }
}

fn otap_to_otlp_logs(otap: OtapArrowRecords) -> ExportLogsServiceRequest {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportLogsRequest(bytes) => {
            ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode logs")
        }
        _ => panic!("expected logs"),
    }
}

fn otap_to_otlp_metrics(otap: OtapArrowRecords) -> ExportMetricsServiceRequest {
    let pdata: OtapPayload = otap.into();
    let otlp_bytes: OtlpProtoBytes = pdata.try_into().expect("convert to OTLP bytes");
    match otlp_bytes {
        OtlpProtoBytes::ExportMetricsRequest(bytes) => {
            ExportMetricsServiceRequest::decode(bytes.as_ref()).expect("decode metrics")
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

#[test]
fn test_simple_split_traces() {
    use crate::testing::equiv::assert_traces_equivalent;

    let traces_otlp = make_traces_otlp();
    let traces_otap = encode_traces(&traces_otlp);
    let traces = RecordsGroup::separate_traces(vec![traces_otap]).unwrap();
    let original_traces = traces.clone();

    // Convert original to OTLP for equivalence checking
    let original_otlp =
        otap_to_otlp_traces(original_traces.clone().into_otap_arrow_records()[0].clone());

    let split = traces.split(NonZeroU64::new(2).unwrap()).unwrap();

    let otap_batches = match split {
        RecordsGroup::Traces(batches) => batches,
        _ => {
            panic!("split returned wrong type of record group. Expecting traces")
        }
    };

    assert_eq!(otap_batches.len(), 2);

    // Verify each split contains the expected number of spans
    let batch0 = OtapArrowRecords::Traces(Traces {
        batches: otap_batches[0].clone(),
    });
    assert_eq!(batch0.get(ArrowPayloadType::Spans).unwrap().num_rows(), 2);

    let batch1 = OtapArrowRecords::Traces(Traces {
        batches: otap_batches[1].clone(),
    });
    assert_eq!(batch1.get(ArrowPayloadType::Spans).unwrap().num_rows(), 2);

    // Merge the batches back together
    let traces_merged = RecordsGroup::separate_traces(vec![batch0, batch1]).unwrap();
    let merged = traces_merged
        .concatenate(Some(NonZeroU64::new(10).unwrap()))
        .unwrap();

    // Verify semantic correctness using equivalence checker
    let merged_otlp = otap_to_otlp_traces(merged.into_otap_arrow_records()[0].clone());
    assert_traces_equivalent(&original_otlp, &merged_otlp);
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
fn test_simple_split_metrics() {
    // Note: metrics equivalence checker not yet implemented, so we just test split/merge mechanics

    let metrics_otlp = make_metrics_otlp();
    let metrics_otap = encode_metrics(&metrics_otlp);
    let metrics = RecordsGroup::separate_metrics(vec![metrics_otap]).unwrap();

    let split = metrics.split(NonZeroU64::new(2).unwrap()).unwrap();

    // Convert to OTAP batches
    let split_batches_vec = split.into_otap_arrow_records();

    // Verify splits have the expected data point counts
    let batch_lengths: Vec<usize> = split_batches_vec
        .iter()
        .map(OtapArrowRecords::batch_length)
        .collect();

    // With max_size=2, we expect splits based on data point counts
    // Total: 5 data points (2+2+1), so we might get [2, 2, 1] or similar
    assert_eq!(
        batch_lengths.iter().sum::<usize>(),
        5,
        "Total data points should be preserved"
    );

    // Merge batches back
    let split_batches = RecordsGroup::separate_metrics(split_batches_vec).unwrap();
    let merged = split_batches
        .concatenate(Some(NonZeroU64::new(10).unwrap()))
        .unwrap();

    // Verify we got a result
    assert_eq!(merged.into_otap_arrow_records()[0].batch_length(), 5);
}

#[test]
fn test_split_metrics_with_varying_datapoint_counts_otlp() {
    use crate::otap::batching::make_output_batches;
    use crate::proto::opentelemetry::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        metrics::v1::{
            Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, metric::Data,
            number_data_point::Value,
        },
        resource::v1::Resource,
    };
    use crate::views::otlp::bytes::metrics::RawMetricsData;
    use prost::Message as ProstMessage;

    // Create OTLP metrics message with:
    // - metric1 (Gauge): 3 data points
    // - metric2 (Sum): 4 data points
    // - metric3 (Gauge): 2 data points
    // Total: 9 data points across 3 metrics

    let otlp_request = ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                attributes: vec![KeyValue::new("service.name", AnyValue::new_string("test"))],
                ..Default::default()
            }),
            scope_metrics: vec![ScopeMetrics {
                scope: Some(InstrumentationScope {
                    name: "test-scope".into(),
                    ..Default::default()
                }),
                schema_url: String::new(),
                metrics: vec![
                    // Metric 1: Gauge with 3 data points
                    Metric {
                        name: "metric1".into(),
                        description: "First metric".into(),
                        unit: "1".into(),
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("a"),
                                    )],
                                    time_unix_nano: 1000,
                                    value: Some(Value::AsDouble(1.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("b"),
                                    )],
                                    time_unix_nano: 2000,
                                    value: Some(Value::AsDouble(2.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("c"),
                                    )],
                                    time_unix_nano: 3000,
                                    value: Some(Value::AsDouble(3.0)),
                                    ..Default::default()
                                },
                            ],
                        })),
                        ..Default::default()
                    },
                    // Metric 2: Sum with 4 data points
                    Metric {
                        name: "metric2".into(),
                        description: "Second metric".into(),
                        unit: "ms".into(),
                        data: Some(Data::Sum(Sum {
                            data_points: vec![
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("d"),
                                    )],
                                    time_unix_nano: 4000,
                                    value: Some(Value::AsDouble(4.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("e"),
                                    )],
                                    time_unix_nano: 5000,
                                    value: Some(Value::AsDouble(5.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("f"),
                                    )],
                                    time_unix_nano: 6000,
                                    value: Some(Value::AsDouble(6.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("g"),
                                    )],
                                    time_unix_nano: 7000,
                                    value: Some(Value::AsDouble(7.0)),
                                    ..Default::default()
                                },
                            ],
                            aggregation_temporality: 1,
                            is_monotonic: true,
                        })),
                        ..Default::default()
                    },
                    // Metric 3: Gauge with 2 data points
                    Metric {
                        name: "metric3".into(),
                        description: "Third metric".into(),
                        unit: "bytes".into(),
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("h"),
                                    )],
                                    time_unix_nano: 8000,
                                    value: Some(Value::AsDouble(8.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "label",
                                        AnyValue::new_string("i"),
                                    )],
                                    time_unix_nano: 9000,
                                    value: Some(Value::AsDouble(9.0)),
                                    ..Default::default()
                                },
                            ],
                        })),
                        ..Default::default()
                    },
                ],
            }],
            ..Default::default()
        }],
    };

    // Encode OTLP to protobuf bytes
    let mut otlp_bytes = Vec::new();
    otlp_request
        .encode(&mut otlp_bytes)
        .expect("Failed to encode OTLP request");

    // Convert OTLP bytes to OTAP using views mechanism
    let metrics_data_view = RawMetricsData::new(&otlp_bytes);
    let otap_batch = crate::encode::encode_metrics_otap_batch(&metrics_data_view)
        .expect("Failed to convert OTLP to OTAP");

    println!(
        "\nOriginal OTAP batch length: {}",
        otap_batch.batch_length()
    );

    // Split with max_output_batch = 5
    // With datapoint counts [3, 4, 2], we expect splitting to occur
    let result = make_output_batches(SignalType::Metrics, vec![otap_batch], NonZeroU64::new(5))
        .expect("make_output_batches failed");

    println!("Number of output batches: {}", result.len());
    for (i, batch) in result.iter().enumerate() {
        let rows = batch.batch_length();
        println!("Batch {}: {} datapoints", i, rows);
    }

    // Verify we got multiple batches (split occurred)
    assert!(
        result.len() > 1,
        "Expected split to occur with max=5 and datapoint counts [3, 4, 2]"
    );

    // Verify total count is preserved (3 + 4 + 2 = 9 datapoints)
    let total_rows: usize = result.iter().map(|b| b.batch_length()).sum();
    assert_eq!(total_rows, 9, "Total datapoints should be preserved");

    // Verify no batch exceeds max size
    for (i, batch) in result.iter().enumerate() {
        assert!(
            batch.batch_length() <= 5,
            "Batch {} has {} datapoints, exceeds max of 5",
            i,
            batch.batch_length()
        );
    }

    println!("✓ Split occurred, total preserved, all batches within limit");
}
