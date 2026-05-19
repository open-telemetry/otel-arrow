// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::OtapArrowRecords;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value as otlp_any_value};
use crate::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    metric, number_data_point,
};
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::views::otlp::bytes::metrics::RawMetricsData;
use otap_df_pdata_views::views::common::{AnyValueView, AttributeView, ValueType};
use otap_df_pdata_views::views::metrics::{
    self as metrics_view, DataView, GaugeView, MetricView, MetricsView, NumberDataPointView,
    ResourceMetricsView, ScopeMetricsView,
};
use prost::Message as ProstMessage;

fn example_gauge_request() -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_owned(),
                    value: Some(AnyValue {
                        value: Some(otlp_any_value::Value::StringValue("demo".to_owned())),
                    }),
                }],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_metrics: vec![ScopeMetrics {
                scope: None,
                metrics: vec![Metric {
                    name: "requests".to_owned(),
                    description: String::new(),
                    unit: "1".to_owned(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: vec![NumberDataPoint {
                            attributes: vec![KeyValue {
                                key: "route".to_owned(),
                                value: Some(AnyValue {
                                    value: Some(otlp_any_value::Value::StringValue(
                                        "/v1/metrics".to_owned(),
                                    )),
                                }),
                            }],
                            start_time_unix_nano: 10,
                            time_unix_nano: 20,
                            exemplars: Vec::new(),
                            flags: 0,
                            value: Some(number_data_point::Value::AsInt(7)),
                        }],
                    })),
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}

fn otlp_bytes(request: &ExportMetricsServiceRequest) -> Vec<u8> {
    let mut bytes = Vec::new();
    request.encode(&mut bytes).unwrap();
    bytes
}

fn string_kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_owned(),
        value: Some(AnyValue {
            value: Some(otlp_any_value::Value::StringValue(value.to_owned())),
        }),
    }
}

fn encode_request_direct(request: &ExportMetricsServiceRequest) -> (Vec<u8>, u64) {
    encode_request_direct_with_compression(request, StefCompression::None)
}

fn encode_request_direct_with_compression(
    request: &ExportMetricsServiceRequest,
    compression: StefCompression,
) -> (Vec<u8>, u64) {
    let bytes = otlp_bytes(request);
    let view = RawMetricsData::new(&bytes);
    encode_metrics_view_with_count_and_compression(&view, compression).unwrap()
}

fn read_uvarint_at(bytes: &[u8], position: &mut usize) -> u64 {
    let mut value = 0_u64;
    let mut shift = 0;
    loop {
        let byte = bytes[*position];
        *position += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return value;
        }
        shift += 7;
    }
}

fn split_go_style_stef_chunks(bytes: &[u8]) -> (&[u8], &[u8], &[u8]) {
    assert!(bytes.starts_with(b"STEF"));

    let mut pos = 4;
    let fixed_header_len = read_uvarint_at(bytes, &mut pos) as usize;
    let fixed_end = pos + fixed_header_len;
    let compression = bytes[pos + 1] & 0b11;

    let var_end = frame_end(bytes, fixed_end, compression);

    (
        &bytes[..fixed_end],
        &bytes[fixed_end..var_end],
        &bytes[var_end..],
    )
}

fn frame_end(bytes: &[u8], start: usize, compression: u8) -> usize {
    let mut pos = start + 1;
    let uncompressed_len = read_uvarint_at(bytes, &mut pos) as usize;
    let frame_len = if compression == 0 {
        uncompressed_len
    } else {
        read_uvarint_at(bytes, &mut pos) as usize
    };
    pos + frame_len
}

fn assert_single_gauge_value(records: &OtapArrowRecords) {
    assert_single_gauge_point(records, 10, 20, metrics_view::Value::Integer(7));
}

fn assert_single_gauge_point(
    records: &OtapArrowRecords,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    value: metrics_view::Value,
) {
    let view = crate::views::otap::metrics::OtapMetricsView::try_from(records).unwrap();
    let resource_metrics = view.resources().next().unwrap();
    let scope_metrics = resource_metrics.scopes().next().unwrap();
    let metric = scope_metrics.metrics().next().unwrap();
    assert_eq!(std::str::from_utf8(metric.name()).unwrap(), "requests");
    assert_eq!(std::str::from_utf8(metric.unit()).unwrap(), "1");

    let data = metric.data().unwrap();
    let gauge = data.as_gauge().unwrap();
    let point = gauge.data_points().next().unwrap();
    assert_eq!(point.start_time_unix_nano(), start_time_unix_nano);
    assert_eq!(point.time_unix_nano(), time_unix_nano);
    assert_eq!(point.value(), Some(value));
}

fn string_attrs<P>(point: &P) -> Vec<(String, String)>
where
    P: NumberDataPointView,
{
    point
        .attributes()
        .map(|attribute| {
            let key = std::str::from_utf8(attribute.key()).unwrap().to_owned();
            let value = attribute.value().unwrap();
            assert_eq!(value.value_type(), ValueType::String);
            let value = std::str::from_utf8(value.as_string().unwrap())
                .unwrap()
                .to_owned();
            (key, value)
        })
        .collect()
}

#[test]
fn encodes_metrics_stream_header_and_schema() {
    let (bytes, record_count) = encode_request_direct(&example_gauge_request());
    assert_eq!(record_count, 1);
    assert!(bytes.starts_with(b"STEF"));
    assert!(
        bytes
            .windows(METRICS_WIRE_SCHEMA.len())
            .any(|window| window == METRICS_WIRE_SCHEMA)
    );
}

#[test]
fn accepts_empty_sum_metric() {
    let mut request = example_gauge_request();
    request.resource_metrics[0].scope_metrics[0].metrics[0].data = Some(metric::Data::Sum(Sum {
        data_points: Vec::new(),
        aggregation_temporality: AggregationTemporality::Cumulative as i32,
        is_monotonic: true,
    }));
    let (bytes, record_count) = encode_request_direct(&request);
    assert_eq!(record_count, 0);
    assert!(bytes.starts_with(b"STEF"));
}

#[test]
fn roundtrips_numeric_gauge_through_direct_otap() {
    let (bytes, encoded_count) = encode_request_direct(&example_gauge_request());
    assert_eq!(encoded_count, 1);

    let (records, decoded_count) = decode_metrics_otap_with_count(&bytes).unwrap();
    assert_eq!(decoded_count, 1);
    assert_eq!(records.num_items(), 1);
    assert_single_gauge_value(&records);
}

#[test]
fn streaming_decoder_accepts_complete_stream_chunk() {
    let (bytes, encoded_count) = encode_request_direct(&example_gauge_request());
    let mut decoder = MetricsStreamDecoder::default();
    let (records, decoded_count) = decoder.decode_chunk(&bytes).unwrap().unwrap();

    assert_eq!(decoded_count, encoded_count);
    assert_single_gauge_value(&records);
}

#[test]
fn streaming_decoder_accepts_go_style_frame_chunks() {
    let (bytes, encoded_count) = encode_request_direct(&example_gauge_request());
    let (fixed_header, var_header, data_frame) = split_go_style_stef_chunks(&bytes);
    let mut decoder = MetricsStreamDecoder::default();

    assert!(decoder.decode_chunk(fixed_header).unwrap().is_none());
    assert!(decoder.decode_chunk(var_header).unwrap().is_none());
    let (records, decoded_count) = decoder.decode_chunk(data_frame).unwrap().unwrap();

    assert_eq!(decoded_count, encoded_count);
    assert_single_gauge_value(&records);
}

#[test]
fn roundtrips_zstd_compressed_metrics_stream() {
    let (bytes, encoded_count) =
        encode_request_direct_with_compression(&example_gauge_request(), StefCompression::Zstd);
    assert_eq!(encoded_count, 1);
    assert_eq!(bytes[6] & 0b11, 1);

    let (records, decoded_count) = decode_metrics_otap_with_count(&bytes).unwrap();
    assert_eq!(decoded_count, 1);
    assert_single_gauge_value(&records);
}

#[test]
fn streaming_decoder_accepts_zstd_go_style_frame_chunks() {
    let (bytes, encoded_count) =
        encode_request_direct_with_compression(&example_gauge_request(), StefCompression::Zstd);
    let (fixed_header, var_header, data_frame) = split_go_style_stef_chunks(&bytes);
    let mut decoder = MetricsStreamDecoder::default();

    assert!(decoder.decode_chunk(fixed_header).unwrap().is_none());
    assert!(decoder.decode_chunk(var_header).unwrap().is_none());
    let (records, decoded_count) = decoder.decode_chunk(data_frame).unwrap().unwrap();

    assert_eq!(decoded_count, encoded_count);
    assert_single_gauge_value(&records);
}

#[test]
fn stream_encoder_reuses_header_and_carries_state_across_frames() {
    let first_otlp_bytes = otlp_bytes(&example_gauge_request());
    let view = RawMetricsData::new(&first_otlp_bytes);
    let mut second_request = example_gauge_request();
    let metric::Data::Gauge(gauge) = second_request.resource_metrics[0].scope_metrics[0].metrics[0]
        .data
        .as_mut()
        .unwrap()
    else {
        panic!("example metric must be a gauge");
    };
    gauge.data_points[0].time_unix_nano = 25;
    gauge.data_points[0].value = Some(number_data_point::Value::AsInt(11));
    let second_otlp_bytes = otlp_bytes(&second_request);
    let second_view = RawMetricsData::new(&second_otlp_bytes);
    let mut encoder = MetricsStreamEncoder::new(StefCompression::Zstd).unwrap();
    let (fixed_header, var_header) = encoder.stream_header_chunks().unwrap().unwrap();

    assert!(encoder.stream_header_chunks().unwrap().is_none());

    let first_frame = encoder.encode_metrics_view_frame(&view).unwrap();
    assert_eq!(first_frame.frame_record_count, 1);
    assert_eq!(first_frame.stream_record_count, 1);
    assert_eq!(encoder.record_count(), 1);

    let second_frame = encoder.encode_metrics_view_frame(&second_view).unwrap();
    assert_eq!(second_frame.frame_record_count, 1);
    assert_eq!(second_frame.stream_record_count, 2);
    assert_eq!(encoder.record_count(), 2);

    let mut complete_stream = Vec::new();
    complete_stream.extend_from_slice(&fixed_header);
    complete_stream.extend_from_slice(&var_header);
    complete_stream.extend_from_slice(&first_frame.bytes);
    complete_stream.extend_from_slice(&second_frame.bytes);
    let (records, decoded_count) = decode_metrics_otap_with_count(&complete_stream).unwrap();
    assert_eq!(decoded_count, 2);
    assert_eq!(records.num_items(), 2);

    let mut decoder = MetricsStreamDecoder::default();
    assert!(decoder.decode_chunk(&fixed_header).unwrap().is_none());
    assert!(decoder.decode_chunk(&var_header).unwrap().is_none());
    let (first_records, first_count) = decoder.decode_chunk(&first_frame.bytes).unwrap().unwrap();
    assert_eq!(first_count, 1);
    assert_single_gauge_value(&first_records);
    let (second_records, second_count) =
        decoder.decode_chunk(&second_frame.bytes).unwrap().unwrap();
    assert_eq!(second_count, 1);
    assert_single_gauge_point(&second_records, 10, 25, metrics_view::Value::Integer(11));
}

#[test]
fn decodes_metrics_stream_directly_to_otap_records() {
    let (bytes, _) = encode_request_direct(&example_gauge_request());
    let (records, record_count) = decode_metrics_otap_with_count(&bytes).unwrap();
    assert_eq!(record_count, 1);
    assert_eq!(records.num_items(), 1);

    let (encoded, encoded_count) = encode_metrics_otap_with_count(&records).unwrap();
    assert_eq!(encoded_count, 1);
    let (decoded_records, decoded_count) = decode_metrics_otap_with_count(&encoded).unwrap();
    assert_eq!(decoded_count, 1);
    assert_single_gauge_value(&decoded_records);
}

#[test]
fn encodes_raw_otlp_metrics_view_directly_to_stef() {
    let otlp_bytes = otlp_bytes(&example_gauge_request());
    let view = RawMetricsData::new(&otlp_bytes);
    let (encoded, record_count) = encode_metrics_view_with_count(&view).unwrap();
    assert_eq!(record_count, 1);

    let (records, decoded_count) = decode_metrics_otap_with_count(&encoded).unwrap();
    assert_eq!(decoded_count, 1);
    assert_single_gauge_value(&records);
}

#[test]
fn encodes_attribute_layout_change_after_repeated_prefix() {
    let mut request = example_gauge_request();
    let metric = &mut request.resource_metrics[0].scope_metrics[0].metrics[0];
    let metric::Data::Gauge(gauge) = metric.data.as_mut().unwrap() else {
        panic!("example metric must be a gauge");
    };
    gauge.data_points = vec![
        NumberDataPoint {
            attributes: vec![string_kv("stable", "one"), string_kv("first", "old")],
            start_time_unix_nano: 10,
            time_unix_nano: 20,
            exemplars: Vec::new(),
            flags: 0,
            value: Some(number_data_point::Value::AsInt(7)),
        },
        NumberDataPoint {
            attributes: vec![string_kv("stable", "one"), string_kv("second", "new")],
            start_time_unix_nano: 10,
            time_unix_nano: 21,
            exemplars: Vec::new(),
            flags: 0,
            value: Some(number_data_point::Value::AsInt(8)),
        },
    ];

    let (encoded, encoded_count) = encode_request_direct(&request);
    assert_eq!(encoded_count, 2);

    let (records, decoded_count) = decode_metrics_otap_with_count(&encoded).unwrap();
    assert_eq!(decoded_count, 2);
    let view = crate::views::otap::metrics::OtapMetricsView::try_from(&records).unwrap();
    let resource_metrics = view.resources().next().unwrap();
    let scope_metrics = resource_metrics.scopes().next().unwrap();
    let metric = scope_metrics.metrics().next().unwrap();
    let data = metric.data().unwrap();
    let gauge = data.as_gauge().unwrap();
    let point = gauge.data_points().nth(1).unwrap();

    assert_eq!(point.value(), Some(metrics_view::Value::Integer(8)));
    assert_eq!(
        string_attrs(&point),
        vec![
            ("stable".to_owned(), "one".to_owned()),
            ("second".to_owned(), "new".to_owned()),
        ]
    );
}
