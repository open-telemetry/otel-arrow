// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(test)]
mod tests {
    use crate::pdata::SpanID;
    use crate::pdata::TraceID;
    use crate::proto::opentelemetry::common::v1::AnyValue;
    use crate::proto::opentelemetry::common::v1::ArrayValue;
    use crate::proto::opentelemetry::common::v1::EntityRef;
    use crate::proto::opentelemetry::common::v1::InstrumentationScope;
    use crate::proto::opentelemetry::common::v1::KeyValue;
    use crate::proto::opentelemetry::common::v1::KeyValueList;
    use crate::proto::opentelemetry::common::v1::any_value::Value;
    use crate::proto::opentelemetry::logs::v1::LogRecord;
    use crate::proto::opentelemetry::logs::v1::LogRecordFlags;
    use crate::proto::opentelemetry::logs::v1::ResourceLogs;
    use crate::proto::opentelemetry::logs::v1::ScopeLogs;
    use crate::proto::opentelemetry::logs::v1::SeverityNumber;
    use crate::proto::opentelemetry::metrics::v1::AggregationTemporality;
    use crate::proto::opentelemetry::metrics::v1::Exemplar;
    use crate::proto::opentelemetry::metrics::v1::Gauge;
    use crate::proto::opentelemetry::metrics::v1::Histogram;
    use crate::proto::opentelemetry::metrics::v1::HistogramDataPoint;
    use crate::proto::opentelemetry::metrics::v1::Metric;
    use crate::proto::opentelemetry::metrics::v1::NumberDataPoint;
    use crate::proto::opentelemetry::metrics::v1::Sum;
    use crate::proto::opentelemetry::metrics::v1::Summary;
    use crate::proto::opentelemetry::metrics::v1::SummaryDataPoint;
    use crate::proto::opentelemetry::metrics::v1::exemplar::Value as ExemplarValue;
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

    #[test]
    fn test_any_value() {
        // Primitives
        assert_eq!(
            AnyValue::new_int(3i64),
            AnyValue {
                value: Some(Value::IntValue(3i64)),
            }
        );
        assert_eq!(
            AnyValue::new_double(3.123),
            AnyValue {
                value: Some(Value::DoubleValue(3.123)),
            }
        );
        assert_eq!(
            AnyValue::new_bool(true),
            AnyValue {
                value: Some(Value::BoolValue(true)),
            }
        );

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
            AnyValue::new_kvlist(&[
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
        let lr1_value = {
            let mut value = LogRecord::default();
            value.time_unix_nano = ts;
            value.severity_number = sev as i32;
            value.event_name = name.into();
            value
        };
        let lr1 = LogRecord::new(ts, sev, name).build();

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

        let lr1_value = {
            let mut value = LogRecord::default();
            value.time_unix_nano = ts;
            value.severity_number = sev as i32;
            value.event_name = name.into();
            value.body = Some(AnyValue::new_string(msg));
            value.severity_text = sevtxt.into();
            value.flags = flags as u32;
            value
        };
        let lr1 = LogRecord::new(ts, sev, name)
            .body(AnyValue::new_string(msg))
            .severity_text(sevtxt)
            .flags(flags)
            .build();

        assert_eq!(lr1, lr1_value);
    }

    #[test]
    fn test_instrumentation_scope_default() {
        let is1 = InstrumentationScope::new("library").build();
        let mut is1_value = InstrumentationScope::default();
        is1_value.name = "library".into();

        assert_eq!(is1, is1_value);
    }

    #[test]
    fn test_instrumentation_scope_options() {
        let kv1 = KeyValue::new("k1", AnyValue::new_string("v1"));
        let kv2 = KeyValue::new("k2", AnyValue::new_int(2));
        let kvs = vec![kv1, kv2];
        let is1 = InstrumentationScope::new("library")
            .version("v1.0")
            .attributes(kvs.clone())
            .dropped_attributes_count(1u32)
            .build();
        let mut is1_value = InstrumentationScope::default();
        is1_value.name = "library".into();
        is1_value.version = "v1.0".into();
        is1_value.attributes = kvs;
        is1_value.dropped_attributes_count = 1u32;

        assert_eq!(is1, is1_value);
    }

    #[test]
    fn test_scope_logs() {
        let kv1 = KeyValue::new("k1", AnyValue::new_string("v1"));
        let kv2 = KeyValue::new("k2", AnyValue::new_int(2));
        let kvs1 = vec![kv1, kv2];
        let body2 = AnyValue::new_string("message text");

        let is1 = InstrumentationScope::new("library").build();

        let lr1 = LogRecord::new(2_000_000_000u64, SeverityNumber::Info, "event1")
            .attributes(kvs1.clone())
            .build();
        let lr2 = LogRecord::new(3_000_000_000u64, SeverityNumber::Info2, "event2")
            .body(body2)
            .build();
        let lrs = vec![lr1, lr2];

        let sl = ScopeLogs::new(is1.clone())
            .log_records(lrs.clone())
            .schema_url("http://schema.opentelemetry.io")
            .build();

        let sl_value = ScopeLogs {
            scope: Some(is1),
            log_records: lrs,
            schema_url: "http://schema.opentelemetry.io".into(),
        };

        assert_eq!(sl, sl_value);
    }

    #[test]
    fn test_entity() {
        let er1 = EntityRef::new("entity")
            .id_keys(&["a".to_string(), "b".to_string(), "c".to_string()])
            .description_keys(&["d".to_string(), "e".to_string(), "f".to_string()])
            .build();

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
        let eref1 = EntityRef::new("etype1").build();
        let eref2 = EntityRef::new("etype2").build();

        let erefs = vec![eref1, eref2];

        let res1 = Resource::new(&[KeyValue::new("k1", AnyValue::new_double(1.23))])
            .entity_refs(erefs.clone())
            .build();
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

        let is1 = InstrumentationScope::new("library").build();

        let lr1 = LogRecord::new(2_000_000_000u64, SeverityNumber::Info, "event1").build();
        let lr2 = LogRecord::new(3_000_000_000u64, SeverityNumber::Info2, "event2").build();
        let lrs = vec![lr1, lr2];

        let sl1 = ScopeLogs::new(is1.clone()).log_records(lrs.clone()).build();
        let sl2 = sl1.clone();
        let sls = vec![sl1, sl2];

        let res = Resource::new(kvs).build();

        let rl = ResourceLogs::new(res.clone())
            .scope_logs(sls.clone())
            .build();

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

        let is1 = InstrumentationScope::new("library").build();

        let tid = TraceID([1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2]);
        let sid = SpanID([1, 2, 1, 2, 1, 2, 1, 2]);
        let psid = SpanID([2, 1, 2, 1, 2, 1, 2, 1]);

        let s1 = Span::new(tid, sid, "myop", 123_000_000_000u64)
            .parent_span_id(psid)
            .attributes(kvs.clone())
            .flags(SpanFlags::ContextHasIsRemoteMask)
            .kind(SpanKind::Server)
            .trace_state("ot=th:0")
            .links(vec![Link::new(tid, sid).build()])
            .events(vec![Event::new("oops", 123_500_000_000u64).build()])
            .end_time_unix_nano(124_000_000_000u64)
            .status(Status::new("oh my!", StatusCode::Error))
            .dropped_attributes_count(1u32)
            .dropped_events_count(1u32)
            .dropped_links_count(1u32)
            .build();

        let s2 = s1.clone();
        let sps = vec![s1, s2];

        let ss1 = ScopeSpans::new(is1.clone()).spans(sps.clone()).build();
        let ss2 = ss1.clone();
        let sss = vec![ss1, ss2];

        let res = Resource::new(vec![]).build();

        let rs1 = ResourceSpans::new(res.clone())
            .scope_spans(sss.clone())
            .build();
        let rs2 = rs1.clone();
        let rss = vec![rs1, rs2];

        let rds = TracesData::new(rss.clone());

        let rds_value = TracesData {
            resource_spans: rss,
        };

        assert_eq!(rds, rds_value);
    }

    #[test]
    fn test_metric_sum() {
	let m1 = Metric::new_sum(
	    "counter",
	    Sum::new(AggregationTemporality::Delta,
		     true,
		     vec![
			 NumberDataPoint::new_int(125_000_000_000u64, 123i64).build(),
			 NumberDataPoint::new_double(125_000_000_000u64, 123f64).build(),
		     ],
	    ),
	).build();

	let m1_value = Metric{
	    name: "counter".to_string(),
	    description: "".to_string(),
	    unit: "".to_string(),
	    metadata: vec![],
	    data: Some(MetricData::Sum(Sum{
		aggregation_temporality: AggregationTemporality::Delta as i32,
		is_monotonic: true,
		data_points: vec![
		    NumberDataPoint{
			attributes: vec![],
			exemplars: vec![],
			flags: 0,
			start_time_unix_nano: 0,
			time_unix_nano: 125_000_000_000u64,
			value: Some(NumberValue::AsInt(123i64)),
		    },
		    NumberDataPoint{
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
	    Gauge::new(
		     vec![
			 NumberDataPoint::new_int(125_000_000_000u64, 123i64).build(),
			 NumberDataPoint::new_double(125_000_000_000u64, 123f64).build(),
		     ],
	    ),
	).build();

	let m1_value = Metric{
	    name: "gauge".to_string(),
	    description: "".to_string(),
	    unit: "".to_string(),
	    metadata: vec![],
	    data: Some(MetricData::Gauge(Gauge{
		data_points: vec![
		    NumberDataPoint{
			attributes: vec![],
			exemplars: vec![],
			flags: 0,
			start_time_unix_nano: 0,
			time_unix_nano: 125_000_000_000u64,
			value: Some(NumberValue::AsInt(123i64)),
		    },
		    NumberDataPoint{
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

	let e1 = Exemplar::new_double(124_500_000_000u64, 10.1)
	    .trace_id(tid)
	    .span_id(sid)
	    .build();
	let e1_value = Exemplar{
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
	    Histogram::new(
		AggregationTemporality::Delta,
		vec![
		    HistogramDataPoint::new(125_000_000_000u64, &[1u64, 2u64, 3u64], &[1.0, 10.0])
			.start_time_unix_nano(124_000_000_000u64)
			.exemplars(vec![
			    Exemplar::new_double(124_500_000_000u64, 10.1)
				.span_id(sid)
				.trace_id(tid)
				.build(),
			])
			.build(),
		    HistogramDataPoint::new(126_000_000_000u64, &[3u64, 2u64, 1u64], &[1.0, 10.0])
			.start_time_unix_nano(125_000_000_000u64)
			.count(100u64)
			.sum(1000.0)
			.min(0.1)
			.max(10.1)
			.build(),
		],
	    ),
	).build();

	let m1_value = Metric{
	    name: "histogram".to_string(),
	    description: "".to_string(),
	    unit: "".to_string(),
	    metadata: vec![],
	    data: Some(MetricData::Histogram(Histogram{
		aggregation_temporality: AggregationTemporality::Delta as i32,
		data_points: vec![
		    HistogramDataPoint{
			attributes: vec![],
			exemplars: vec![
			    Exemplar{
				filtered_attributes: vec![],
				span_id: sid.0.to_vec(),
				trace_id: tid.0.to_vec(),
				time_unix_nano: 124_500_000_000u64,
				value: Some(ExemplarValue::AsDouble(10.1)),
			    },
			],
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
		    HistogramDataPoint{
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
	    Summary::new(
		vec![
		    SummaryDataPoint::new(125_000_000_000u64, vec![
			    ValueAtQuantile::new(0.1, 0.1),
			    ValueAtQuantile::new(0.5, 2.1),
			    ValueAtQuantile::new(1.0, 10.1),
		    ])
			.start_time_unix_nano(124_000_000_000u64)
			.count(100u64)
			.sum(1000.0)
			.build(),
		    SummaryDataPoint::new(126_000_000_000u64, vec![
			    ValueAtQuantile::new(0.1, 0.5),
			    ValueAtQuantile::new(0.5, 2.5),
			    ValueAtQuantile::new(1.0, 10.5),
		    ])
			.start_time_unix_nano(124_000_000_000u64)
			.count(200u64)
			.sum(2000.0)
			.build(),
		],
	    ),
	).build();

	let m1_value = Metric{
	    name: "summary".to_string(),
	    description: "".to_string(),
	    unit: "".to_string(),
	    metadata: vec![],
	    data: Some(MetricData::Summary(Summary{
		data_points: vec![
		    SummaryDataPoint{
			attributes: vec![],
			flags: 0,
			start_time_unix_nano: 124_000_000_000u64,
			time_unix_nano: 125_000_000_000u64,
			quantile_values: vec![
			    ValueAtQuantile{
				quantile: 0.1,
				value: 0.1,
			    },
			    ValueAtQuantile{
				quantile: 0.5,
				value: 2.1,
			    },
			    ValueAtQuantile{
				quantile: 1.0,
				value: 10.1,
			    },
			],
			count: 100,
			sum: 1000.0,
		    },
		    SummaryDataPoint{
			attributes: vec![],
			flags: 0,
			start_time_unix_nano: 124_000_000_000u64,
			time_unix_nano: 126_000_000_000u64,
			quantile_values: vec![
			    ValueAtQuantile{
				quantile: 0.1,
				value: 0.5,
			    },
			    ValueAtQuantile{
				quantile: 0.5,
				value: 2.5,
			    },
			    ValueAtQuantile{
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
    
}
