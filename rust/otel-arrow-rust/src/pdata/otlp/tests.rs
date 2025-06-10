// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
use crate::pdata::SpanID;
use crate::pdata::TraceID;
use crate::pdata::otlp::ItemCounter;
use crate::pdata::otlp::LogsVisitor;
#[cfg(test)]
use crate::pdata::otlp::PrecomputedSizes;
use crate::proto::opentelemetry::common::v1::AnyValue;
use crate::proto::opentelemetry::common::v1::ArrayValue;
use crate::proto::opentelemetry::common::v1::EntityRef;
use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::proto::opentelemetry::common::v1::KeyValue;
use crate::proto::opentelemetry::common::v1::KeyValueList;
use crate::proto::opentelemetry::common::v1::any_value::Value;
use crate::proto::opentelemetry::logs::v1::LogRecord;
use crate::proto::opentelemetry::logs::v1::LogRecordFlags;
use crate::proto::opentelemetry::logs::v1::LogsData;
use crate::proto::opentelemetry::logs::v1::LogsDataMessageAdapter;
use crate::proto::opentelemetry::logs::v1::ResourceLogs;
use crate::proto::opentelemetry::logs::v1::ScopeLogs;
use crate::proto::opentelemetry::logs::v1::SeverityNumber;
use crate::proto::opentelemetry::metrics::v1::AggregationTemporality;
use crate::proto::opentelemetry::metrics::v1::Exemplar;
use crate::proto::opentelemetry::metrics::v1::ExponentialHistogram;
use crate::proto::opentelemetry::metrics::v1::ExponentialHistogramDataPoint;
use crate::proto::opentelemetry::metrics::v1::Gauge;
use crate::proto::opentelemetry::metrics::v1::Histogram;
use crate::proto::opentelemetry::metrics::v1::HistogramDataPoint;
use crate::proto::opentelemetry::metrics::v1::Metric;
use crate::proto::opentelemetry::metrics::v1::NumberDataPoint;
use crate::proto::opentelemetry::metrics::v1::Sum;
use crate::proto::opentelemetry::metrics::v1::Summary;
use crate::proto::opentelemetry::metrics::v1::SummaryDataPoint;
use crate::proto::opentelemetry::metrics::v1::exemplar::Value as ExemplarValue;
use crate::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
use crate::proto::opentelemetry::metrics::v1::metric::Data as MetricData;
use crate::proto::opentelemetry::metrics::v1::number_data_point::Value as NumberValue;
use crate::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::proto::opentelemetry::trace::v1::ResourceSpans;
use crate::proto::opentelemetry::trace::v1::ScopeSpans;
use crate::proto::opentelemetry::trace::v1::Span;
use crate::proto::opentelemetry::trace::v1::SpanFlags;
use crate::proto::opentelemetry::trace::v1::Status;
use crate::proto::opentelemetry::trace::v1::TracesData;
use crate::proto::opentelemetry::trace::v1::span::Event;
use crate::proto::opentelemetry::trace::v1::span::Link;
use crate::proto::opentelemetry::trace::v1::span::SpanKind;
use crate::proto::opentelemetry::trace::v1::status::StatusCode;

#[cfg(test)]
use crate::proto::opentelemetry::common::v1::{
    AnyValueEncodedLen, AnyValueMessageAdapter, KeyValueEncodedLen, KeyValueMessageAdapter,
};
#[cfg(test)]
use crate::proto::opentelemetry::common::v1::{
    InstrumentationScopeEncodedLen, InstrumentationScopeMessageAdapter,
};
#[cfg(test)]
use crate::proto::opentelemetry::logs::v1::{
    LogRecordEncodedLen, LogRecordMessageAdapter, LogsDataEncodedLen,
};
#[cfg(test)]
use crate::proto::opentelemetry::metrics::v1::{MetricsDataEncodedLen, MetricsDataMessageAdapter};
#[cfg(test)]
use crate::proto::opentelemetry::resource::v1::{ResourceEncodedLen, ResourceMessageAdapter};
#[cfg(test)]
use crate::proto::opentelemetry::trace::v1::{
    ResourceSpansEncodedLen, ResourceSpansMessageAdapter,
};
#[cfg(test)]
use crate::proto::opentelemetry::trace::v1::{SpanEncodedLen, SpanMessageAdapter};
#[cfg(test)]
use crate::proto::opentelemetry::trace::v1::{TracesDataEncodedLen, TracesDataMessageAdapter};

#[test]
fn test_any_value() {
    // Primitives
    assert_eq!(AnyValue::new_int(3i64), AnyValue {
        value: Some(Value::IntValue(3i64)),
    });
    assert_eq!(AnyValue::new_double(3.123), AnyValue {
        value: Some(Value::DoubleValue(3.123)),
    });
    assert_eq!(AnyValue::new_bool(true), AnyValue {
        value: Some(Value::BoolValue(true)),
    });

    // String
    let xyz = "xyz".to_string();
    let xyz_value = AnyValue {
        value: Some(Value::StringValue(xyz.to_string())),
    };
    assert_eq!(AnyValue::new_string("xyz"), xyz_value);
    assert_eq!(AnyValue::new_string(&xyz), xyz_value);
    assert_eq!(AnyValue::new_string(xyz), xyz_value);

    // Bytes
    let hello: Vec<u8> = [104, 101, 108, 108, 111].to_vec();
    let hello_value = AnyValue {
        value: Some(Value::BytesValue(b"hello".to_vec())),
    };
    assert_eq!(AnyValue::new_bytes(hello.as_slice()), hello_value);
    assert_eq!(AnyValue::new_bytes(hello), hello_value);

    // Kvlist
    let kvs = vec![
        KeyValue::new("k1", AnyValue::new_string("s1")),
        KeyValue::new("k2", AnyValue::new_double(2.0)),
    ];
    let kvs_value = AnyValue {
        value: Some(Value::KvlistValue(KeyValueList {
            values: kvs.clone(),
        })),
    };

    assert_eq!(AnyValue::new_kvlist(kvs), kvs_value);
    assert_eq!(
        AnyValue::new_kvlist(vec![
            KeyValue::new("k1", AnyValue::new_string("s1")),
            KeyValue::new("k2", AnyValue::new_double(2.0)),
        ]),
        kvs_value
    );

    // Array
    let vals = vec![AnyValue::new_string("s1"), AnyValue::new_double(2.0)];
    let vals_value = AnyValue {
        value: Some(Value::ArrayValue(ArrayValue {
            values: vals.clone(),
        })),
    };

    assert_eq!(AnyValue::new_array(vals), vals_value);
    assert_eq!(
        AnyValue::new_array(vec![AnyValue::new_string("s1"), AnyValue::new_double(2.0),]),
        vals_value
    );
}

#[test]
fn test_key_value() {
    let k1 = "k1".to_string();
    let k2 = "k2".to_string();
    let v1 = AnyValue::new_string("v1");
    let v2 = AnyValue::new_double(1.23);

    let kv1_value = KeyValue {
        key: k1.clone(),
        value: Some(v1.clone()),
    };
    let kv2_value = KeyValue {
        key: k2.clone(),
        value: Some(v2.clone()),
    };

    assert_eq!(KeyValue::new("k1", v1.clone()), kv1_value);
    assert_eq!(KeyValue::new(k1.clone(), v1), kv1_value);

    assert_eq!(KeyValue::new("k2", v2.clone()), kv2_value);
    assert_eq!(KeyValue::new(k2, v2), kv2_value);
}

#[test]
fn test_log_record_required() {
    let name = "my_log";
    let ts = 1_000_000_000_000u64;
    let sev = SeverityNumber::Info;
    let lr1_value = LogRecord {
        time_unix_nano: ts,
        severity_number: sev as i32,
        event_name: name.into(),
        ..Default::default()
    };
    let lr1 = LogRecord::new(ts, sev, name);

    assert_eq!(lr1, lr1_value);
}

#[test]
fn test_log_record_required_all() {
    let name = "my_log";
    let sevtxt = "not on fire";
    let msg = "message in a bottle";
    let ts = 1_000_000_000_000u64;
    let sev = SeverityNumber::Info;
    let flags = LogRecordFlags::TraceFlagsMask;

    let lr1_value = LogRecord {
        time_unix_nano: ts,
        severity_number: sev as i32,
        event_name: name.into(),
        body: Some(AnyValue::new_string(msg)),
        severity_text: sevtxt.into(),
        flags: flags as u32,
        ..Default::default()
    };
    let lr1 = LogRecord::build(ts, sev, name)
        .body(AnyValue::new_string(msg))
        .severity_text(sevtxt)
        .flags(flags)
        .finish();

    assert_eq!(lr1, lr1_value);
}

#[test]
fn test_instrumentation_scope_default() {
    let is1 = InstrumentationScope::new("library");
    let is1_value = InstrumentationScope {
        name: "library".into(),
        ..Default::default()
    };
    assert_eq!(is1, is1_value);
}

#[test]
fn test_instrumentation_scope_options() {
    let kv1 = KeyValue::new("k1", AnyValue::new_string("v1"));
    let kv2 = KeyValue::new("k2", AnyValue::new_int(2));
    let kvs = vec![kv1, kv2];
    let is1 = InstrumentationScope::build("library")
        .version("v1.0")
        .attributes(kvs.clone())
        .dropped_attributes_count(1u32)
        .finish();
    let is1_value = InstrumentationScope {
        name: "library".into(),
        version: "v1.0".into(),
        attributes: kvs,
        dropped_attributes_count: 1u32,
    };

    assert_eq!(is1, is1_value);
}

#[test]
fn test_scope_logs() {
    let kv1 = KeyValue::new("k1", AnyValue::new_string("v1"));
    let kv2 = KeyValue::new("k2", AnyValue::new_int(2));
    let kvs1 = vec![kv1, kv2];
    let body2 = AnyValue::new_string("message text");

    let is1 = InstrumentationScope::new("library");

    let lr1 = LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
        .attributes(kvs1.clone())
        .finish();
    let lr2 = LogRecord::build(3_000_000_000u64, SeverityNumber::Info2, "event2")
        .body(body2)
        .finish();
    let lrs = vec![lr1, lr2];

    let sl = ScopeLogs::build(is1.clone())
        .log_records(lrs.clone())
        .schema_url("http://schema.opentelemetry.io")
        .finish();

    let sl_value = ScopeLogs {
        scope: Some(is1),
        log_records: lrs,
        schema_url: "http://schema.opentelemetry.io".into(),
    };

    assert_eq!(sl, sl_value);
}

#[test]
fn test_entity() {
    let er1 = EntityRef::build("entity")
        .id_keys(&["a".to_string(), "b".to_string(), "c".to_string()])
        .description_keys(&["d".to_string(), "e".to_string(), "f".to_string()])
        .finish();

    let er1_value = EntityRef {
        r#type: "entity".into(),
        id_keys: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        description_keys: vec!["d".to_string(), "e".to_string(), "f".to_string()],
        schema_url: "".to_string(),
    };

    assert_eq!(er1, er1_value);
}

#[test]
fn test_resource() {
    let eref1 = EntityRef::new("etype1");
    let eref2 = EntityRef::new("etype2");

    let erefs = vec![eref1, eref2];

    let res1 = Resource::build(&[KeyValue::new("k1", AnyValue::new_double(1.23))])
        .entity_refs(erefs.clone())
        .finish();
    let res1_value = Resource {
        attributes: vec![KeyValue::new("k1", AnyValue::new_double(1.23))],
        entity_refs: erefs,
        dropped_attributes_count: 0,
    };

    assert_eq!(res1, res1_value);
}

#[test]
fn test_resource_logs() {
    let kv1 = KeyValue::new("k1", AnyValue::new_string("v1"));
    let kv2 = KeyValue::new("k2", AnyValue::new_int(2));
    let kvs = vec![kv1, kv2];

    let is1 = InstrumentationScope::new("library");

    let lr1 = LogRecord::new(2_000_000_000u64, SeverityNumber::Info, "event1");
    let lr2 = LogRecord::new(3_000_000_000u64, SeverityNumber::Info2, "event2");
    let lrs = vec![lr1, lr2];

    let sl1 = ScopeLogs::build(is1.clone())
        .log_records(lrs.clone())
        .finish();
    let sl2 = sl1.clone();
    let sls = vec![sl1, sl2];

    let res = Resource::new(kvs);

    let rl = ResourceLogs::build(res.clone())
        .scope_logs(sls.clone())
        .finish();

    let rl_value = ResourceLogs {
        resource: Some(res),
        scope_logs: sls,
        schema_url: "".into(),
    };

    assert_eq!(rl, rl_value);
}

#[test]
fn test_resource_spans() {
    let kv1 = KeyValue::new("k1", AnyValue::new_string("v1"));
    let kv2 = KeyValue::new("k2", AnyValue::new_int(2));
    let kvs = vec![kv1, kv2];

    let is1 = InstrumentationScope::new("library");

    let tid: TraceID = [1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2].into();
    let sid: SpanID = [1, 2, 1, 2, 1, 2, 1, 2].into();
    let psid: SpanID = [2, 1, 2, 1, 2, 1, 2, 1].into();

    let s1 = Span::build(tid, sid, "myop", 123_000_000_000u64)
        .parent_span_id(psid)
        .attributes(kvs.clone())
        .flags(SpanFlags::ContextHasIsRemoteMask)
        .kind(SpanKind::Server)
        .trace_state("ot=th:0")
        .links(vec![Link::new(tid, sid)])
        .events(vec![Event::new("oops", 123_500_000_000u64)])
        .end_time_unix_nano(124_000_000_000u64)
        .status(Status::new("oh my!", StatusCode::Error))
        .dropped_attributes_count(1u32)
        .dropped_events_count(1u32)
        .dropped_links_count(1u32)
        .finish();

    let s2 = s1.clone();
    let sps = vec![s1, s2];

    let ss1 = ScopeSpans::build(is1.clone()).spans(sps.clone()).finish();
    let ss2 = ss1.clone();
    let sss = vec![ss1, ss2];

    let res = Resource::new(vec![]);

    let rs1 = ResourceSpans::build(res.clone())
        .scope_spans(sss.clone())
        .finish();
    let rs2 = rs1.clone();
    let rss = vec![rs1, rs2];

    let rds = TracesData::new(rss.clone());

    let rds_value = TracesData {
        resource_spans: rss,
    };

    assert_eq!(rds, rds_value);
    
    // Test that our generated pdata_size() method matches prost's encoded_len()
    assert_eq!(rds.pdata_size(), rds.encoded_len());
    assert_eq!(rds_value.pdata_size(), rds_value.encoded_len());
}

#[test]
fn test_metric_sum() {
    let m1 = Metric::new_sum(
        "counter",
        Sum::new(AggregationTemporality::Delta, true, vec![
            NumberDataPoint::new_int(125_000_000_000u64, 123i64),
            NumberDataPoint::new_double(125_000_000_000u64, 123f64),
        ]),
    );

    let m1_value = Metric {
        name: "counter".to_string(),
        description: "".to_string(),
        unit: "".to_string(),
        metadata: vec![],
        data: Some(MetricData::Sum(Sum {
            aggregation_temporality: AggregationTemporality::Delta as i32,
            is_monotonic: true,
            data_points: vec![
                NumberDataPoint {
                    attributes: vec![],
                    exemplars: vec![],
                    flags: 0,
                    start_time_unix_nano: 0,
                    time_unix_nano: 125_000_000_000u64,
                    value: Some(NumberValue::AsInt(123i64)),
                },
                NumberDataPoint {
                    attributes: vec![],
                    exemplars: vec![],
                    flags: 0,
                    start_time_unix_nano: 0,
                    time_unix_nano: 125_000_000_000u64,
                    value: Some(NumberValue::AsDouble(123f64)),
                },
            ],
        })),
    };

    assert_eq!(m1, m1_value);
}

#[test]
fn test_metric_gauge() {
    let m1 = Metric::new_gauge(
        "gauge",
        Gauge::new(vec![
            NumberDataPoint::new_int(125_000_000_000u64, 123i64),
            NumberDataPoint::new_double(125_000_000_000u64, 123f64),
        ]),
    );

    let m1_value = Metric {
        name: "gauge".to_string(),
        description: "".to_string(),
        unit: "".to_string(),
        metadata: vec![],
        data: Some(MetricData::Gauge(Gauge {
            data_points: vec![
                NumberDataPoint {
                    attributes: vec![],
                    exemplars: vec![],
                    flags: 0,
                    start_time_unix_nano: 0,
                    time_unix_nano: 125_000_000_000u64,
                    value: Some(NumberValue::AsInt(123i64)),
                },
                NumberDataPoint {
                    attributes: vec![],
                    exemplars: vec![],
                    flags: 0,
                    start_time_unix_nano: 0,
                    time_unix_nano: 125_000_000_000u64,
                    value: Some(NumberValue::AsDouble(123f64)),
                },
            ],
        })),
    };

    assert_eq!(m1, m1_value);
}

#[test]
fn test_exemplar() {
    let tid = TraceID([1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2]);
    let sid = SpanID([1, 2, 1, 2, 1, 2, 1, 2]);

    let e1 = Exemplar::build_double(124_500_000_000u64, 10.1)
        .trace_id(tid)
        .span_id(sid)
        .finish();
    let e1_value = Exemplar {
        filtered_attributes: vec![],
        trace_id: tid.0.to_vec(),
        span_id: sid.0.to_vec(),
        time_unix_nano: 124_500_000_000u64,
        value: Some(ExemplarValue::AsDouble(10.1)),
    };

    assert_eq!(e1, e1_value);
}

#[test]
fn test_metric_histogram() {
    let tid = TraceID([1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2]);
    let sid = SpanID([1, 2, 1, 2, 1, 2, 1, 2]);

    let m1 = Metric::new_histogram(
        "histogram",
        Histogram::new(AggregationTemporality::Delta, vec![
            HistogramDataPoint::build(125_000_000_000u64, [1u64, 2u64, 3u64], [1.0, 10.0])
                .start_time_unix_nano(124_000_000_000u64)
                .exemplars(vec![
                    Exemplar::build_double(124_500_000_000u64, 10.1)
                        .span_id(sid)
                        .trace_id(tid)
                        .finish(),
                ])
                .finish(),
            HistogramDataPoint::build(126_000_000_000u64, [3u64, 2u64, 1u64], [1.0, 10.0])
                .start_time_unix_nano(125_000_000_000u64)
                .count(100u64)
                .sum(1000.0)
                .min(0.1)
                .max(10.1)
                .finish(),
        ]),
    );

    let m1_value = Metric {
        name: "histogram".to_string(),
        description: "".to_string(),
        unit: "".to_string(),
        metadata: vec![],
        data: Some(MetricData::Histogram(Histogram {
            aggregation_temporality: AggregationTemporality::Delta as i32,
            data_points: vec![
                HistogramDataPoint {
                    attributes: vec![],
                    exemplars: vec![Exemplar {
                        filtered_attributes: vec![],
                        span_id: sid.0.to_vec(),
                        trace_id: tid.0.to_vec(),
                        time_unix_nano: 124_500_000_000u64,
                        value: Some(ExemplarValue::AsDouble(10.1)),
                    }],
                    flags: 0,
                    start_time_unix_nano: 124_000_000_000u64,
                    time_unix_nano: 125_000_000_000u64,
                    bucket_counts: vec![1u64, 2u64, 3u64],
                    explicit_bounds: vec![1.0, 10.0],
                    count: 0,
                    sum: None,
                    min: None,
                    max: None,
                },
                HistogramDataPoint {
                    attributes: vec![],
                    exemplars: vec![],
                    flags: 0,
                    start_time_unix_nano: 125_000_000_000u64,
                    time_unix_nano: 126_000_000_000u64,
                    bucket_counts: vec![3u64, 2u64, 1u64],
                    explicit_bounds: vec![1.0, 10.0],
                    count: 100,
                    sum: Some(1000.0),
                    min: Some(0.1),
                    max: Some(10.1),
                },
            ],
        })),
    };

    assert_eq!(m1, m1_value);
}

#[test]
fn test_metric_summary() {
    let m1 = Metric::new_summary(
        "summary",
        Summary::new(vec![
            SummaryDataPoint::build(125_000_000_000u64, vec![
                ValueAtQuantile::new(0.1, 0.1),
                ValueAtQuantile::new(0.5, 2.1),
                ValueAtQuantile::new(1.0, 10.1),
            ])
            .start_time_unix_nano(124_000_000_000u64)
            .count(100u64)
            .sum(1000.0)
            .finish(),
            SummaryDataPoint::build(126_000_000_000u64, vec![
                ValueAtQuantile::new(0.1, 0.5),
                ValueAtQuantile::new(0.5, 2.5),
                ValueAtQuantile::new(1.0, 10.5),
            ])
            .start_time_unix_nano(124_000_000_000u64)
            .count(200u64)
            .sum(2000.0)
            .finish(),
        ]),
    );

    let m1_value = Metric {
        name: "summary".to_string(),
        description: "".to_string(),
        unit: "".to_string(),
        metadata: vec![],
        data: Some(MetricData::Summary(Summary {
            data_points: vec![
                SummaryDataPoint {
                    attributes: vec![],
                    flags: 0,
                    start_time_unix_nano: 124_000_000_000u64,
                    time_unix_nano: 125_000_000_000u64,
                    quantile_values: vec![
                        ValueAtQuantile {
                            quantile: 0.1,
                            value: 0.1,
                        },
                        ValueAtQuantile {
                            quantile: 0.5,
                            value: 2.1,
                        },
                        ValueAtQuantile {
                            quantile: 1.0,
                            value: 10.1,
                        },
                    ],
                    count: 100,
                    sum: 1000.0,
                },
                SummaryDataPoint {
                    attributes: vec![],
                    flags: 0,
                    start_time_unix_nano: 124_000_000_000u64,
                    time_unix_nano: 126_000_000_000u64,
                    quantile_values: vec![
                        ValueAtQuantile {
                            quantile: 0.1,
                            value: 0.5,
                        },
                        ValueAtQuantile {
                            quantile: 0.5,
                            value: 2.5,
                        },
                        ValueAtQuantile {
                            quantile: 1.0,
                            value: 10.5,
                        },
                    ],
                    count: 200,
                    sum: 2000.0,
                },
            ],
        })),
    };

    assert_eq!(m1, m1_value);
}

#[test]
fn test_metric_exponential_histogram() {
    let m1 = Metric::new_exponential_histogram(
        "exp_histogram",
        ExponentialHistogram::new(AggregationTemporality::Delta, vec![
            ExponentialHistogramDataPoint::build(
                125_000_000_000u64,
                7,
                Buckets::new(1, vec![3, 4, 5]),
            )
            .start_time_unix_nano(124_000_000_000u64)
            .count(17u64)
            .zero_count(2u64)
            .negative(Buckets::new(0, vec![1, 2]))
            .finish(),
        ]),
    );

    let m1_value = Metric {
        name: "exp_histogram".to_string(),
        description: "".to_string(),
        unit: "".to_string(),
        metadata: vec![],
        data: Some(MetricData::ExponentialHistogram(ExponentialHistogram {
            aggregation_temporality: AggregationTemporality::Delta as i32,
            data_points: vec![ExponentialHistogramDataPoint {
                attributes: vec![],
                exemplars: vec![],
                flags: 0,
                start_time_unix_nano: 124_000_000_000u64,
                time_unix_nano: 125_000_000_000u64,
                count: 17,
                sum: None,
                min: None,
                max: None,
                scale: 7,
                positive: Some(Buckets {
                    offset: 1,
                    bucket_counts: vec![3, 4, 5],
                }),
                negative: Some(Buckets {
                    offset: 0,
                    bucket_counts: vec![1, 2],
                }),
                zero_count: 2,
                zero_threshold: 0.0,
            }],
        })),
    };

    assert_eq!(m1, m1_value);
}

#[test]
fn test_logs_item_count() {
    let ld = LogsData::new(vec![
        ResourceLogs::build(Resource::new(vec![]))
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope::new("test0"))
                    .log_records(vec![
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                    ])
                    .finish(),
                ScopeLogs::build(InstrumentationScope::new("test1"))
                    .log_records(vec![
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                    ])
                    .finish(),
            ])
            .finish(),
    ]);

    let ic = ItemCounter::new().visit_logs(&LogsDataMessageAdapter::new(&ld));
    assert_eq!(20, ic);
}

#[test]
fn test_precomputed_sizes_varint_len() {
    use crate::pdata::otlp::PrecomputedSizes;

    assert_eq!(PrecomputedSizes::varint_len(0), 1);
    assert_eq!(PrecomputedSizes::varint_len(127), 1);
    assert_eq!(PrecomputedSizes::varint_len(128), 2);
    assert_eq!(PrecomputedSizes::varint_len(16383), 2);
    assert_eq!(PrecomputedSizes::varint_len(16384), 3);
}
