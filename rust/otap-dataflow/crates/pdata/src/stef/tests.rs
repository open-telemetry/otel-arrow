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
    let bytes = otlp_bytes(request);
    let view = RawMetricsData::new(&bytes);
    encode_metrics_view_with_count(&view).unwrap()
}

fn assert_single_gauge_value(records: &OtapArrowRecords) {
    let view = crate::views::otap::metrics::OtapMetricsView::try_from(records).unwrap();
    let resource_metrics = view.resources().next().unwrap();
    let scope_metrics = resource_metrics.scopes().next().unwrap();
    let metric = scope_metrics.metrics().next().unwrap();
    assert_eq!(std::str::from_utf8(metric.name()).unwrap(), "requests");
    assert_eq!(std::str::from_utf8(metric.unit()).unwrap(), "1");

    let data = metric.data().unwrap();
    let gauge = data.as_gauge().unwrap();
    let point = gauge.data_points().next().unwrap();
    assert_eq!(point.start_time_unix_nano(), 10);
    assert_eq!(point.time_unix_nano(), 20);
    assert_eq!(point.value(), Some(metrics_view::Value::Integer(7)));
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
