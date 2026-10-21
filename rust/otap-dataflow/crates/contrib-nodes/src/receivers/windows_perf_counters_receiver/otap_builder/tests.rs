// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::config::RuntimeConfig;
use super::super::model::{SamplePoint, scale_integer};
use super::*;
use arrow::array::{Array, TimestampNanosecondArray};
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, metric, number_data_point,
};
use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
use otel_arrow_dfe_pdata_codec::OtapPayload;
use prost::Message;
use std::sync::Arc;

fn counter(name: &str, kind: MetricKind) -> CounterConfig {
    CounterConfig {
        path: format!(r"\Object\{name}"),
        name: name.to_owned(),
        unit: "By".to_owned(),
        description: Arc::from(format!("Description for {name}.")),
        kind,
        attributes: Arc::new(BTreeMap::new()),
        scale_power10: 0,
    }
}

fn point(counter_index: usize, value: SampleValue) -> SamplePoint {
    SamplePoint {
        counter_index,
        value,
    }
}

fn sample(start: i64, timestamp: i64, points: Vec<SamplePoint>) -> Sample {
    Sample {
        start_time_unix_nano: start,
        timestamp_unix_nano: timestamp,
        points,
        failures: Vec::new(),
    }
}

fn decode(records: OtapArrowRecords) -> ExportMetricsServiceRequest {
    let bytes: OtlpProtoBytes = OtapPayload::from_otap(records)
        .try_into_with_default()
        .unwrap();
    ExportMetricsServiceRequest::decode(bytes.as_bytes()).unwrap()
}

fn attributes(attrs: &[KeyValue]) -> BTreeMap<&str, &str> {
    attrs
        .iter()
        .map(|attr| {
            let Some(any_value::Value::StringValue(value)) =
                attr.value.as_ref().and_then(|value| value.value.as_ref())
            else {
                panic!("expected a string attribute");
            };
            (attr.key.as_str(), value.as_str())
        })
        .collect()
}

/// Scenario: A collection contains integer and double Gauges and cumulative non-monotonic Sums.
/// Guarantees: Decoded output retains metadata, exact values, timestamps and cumulative non-monotonic Sum shape.
#[test]
fn projects_metric_kinds_and_numeric_types() {
    let start = 1_788_400_000_000_000_000;
    let timestamp = start + 123_456_789;
    let counters = [
        counter("integer_gauge", MetricKind::Gauge),
        counter("double_gauge", MetricKind::Gauge),
        counter("integer_sum", MetricKind::Sum),
        counter("double_sum", MetricKind::Sum),
    ];
    let values = [
        Number::Integer(9_007_199_254_740_993),
        Number::Double(250.5),
        Number::Integer(-42),
        scale_integer(42, 3).unwrap(),
    ];
    let records = into_otap(
        &counters,
        sample(
            start,
            timestamp,
            values
                .iter()
                .enumerate()
                .map(|(index, value)| point(index, SampleValue::Value(*value)))
                .collect(),
        ),
    )
    .unwrap()
    .unwrap();
    let starts = records
        .get(ArrowPayloadType::NumberDataPoints)
        .unwrap()
        .column_by_name("start_time_unix_nano")
        .unwrap()
        .as_any()
        .downcast_ref::<TimestampNanosecondArray>()
        .unwrap();
    assert!(starts.is_null(0));
    assert!(starts.is_null(1));
    assert_eq!(starts.value(2), start);
    assert_eq!(starts.value(3), start);

    let decoded = decode(records);
    assert_eq!(decoded.resource_metrics.len(), 1);
    let resource = &decoded.resource_metrics[0];
    assert_eq!(
        attributes(&resource.resource.as_ref().unwrap().attributes),
        BTreeMap::from([("os.type", "windows")])
    );
    assert_eq!(resource.scope_metrics.len(), 1);
    let scope = &resource.scope_metrics[0];
    let metadata = scope.scope.as_ref().unwrap();
    assert_eq!(
        metadata.name,
        "otel-arrow-dfe-contrib-nodes/windows_perf_counters"
    );
    assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(scope.metrics.len(), counters.len());
    for ((metric, counter), value) in scope.metrics.iter().zip(&counters).zip(values) {
        assert_eq!(metric.name, counter.name);
        assert_eq!(metric.description, counter.description.as_ref());
        assert_eq!(metric.unit, counter.unit);
        let points = match (counter.kind, metric.data.as_ref().unwrap()) {
            (MetricKind::Gauge, metric::Data::Gauge(gauge)) => &gauge.data_points,
            (MetricKind::Sum, metric::Data::Sum(sum)) => {
                assert_eq!(
                    sum.aggregation_temporality,
                    AggregationTemporality::Cumulative as i32
                );
                assert!(!sum.is_monotonic);
                &sum.data_points
            }
            _ => panic!("incorrect metric shape"),
        };
        assert_eq!(points.len(), 1);
        let point = &points[0];
        assert_eq!(point.time_unix_nano, timestamp as u64);
        assert_eq!(
            point.start_time_unix_nano,
            if counter.kind == MetricKind::Gauge {
                0
            } else {
                start as u64
            }
        );
        assert_eq!(point.flags, 0);
        assert_eq!(
            point.value,
            Some(match value {
                Number::Integer(value) => number_data_point::Value::AsInt(value),
                Number::Double(value) => number_data_point::Value::AsDouble(value),
            })
        );
        assert_eq!(
            attributes(&point.attributes),
            BTreeMap::from([("windows.perf_counter.path", counter.path.as_str())])
        );
    }
}

/// Scenario: Validated configuration maps exact counters to a shared metric with static attributes.
/// Guarantees: Observed points share one metric and keep their own path and attributes despite omissions and reordering.
#[test]
fn projects_shared_metric_with_attributes_and_omissions() {
    let config = RuntimeConfig::from_json(&serde_json::json!({
        "metrics": {
            "windows.processor.time": {
                "description": "Processor time", "unit": "%", "gauge": {}
            },
            "windows.process.private": {
                "description": "Private bytes", "unit": "By",
                "sum": {"aggregation": "cumulative", "monotonic": false}
            }
        },
        "perfcounters": [
            {"object": "Processor", "instances": "_Total", "counters": [
                {"name": "% Processor Time", "metric": "windows.processor.time",
                 "attributes": {"state": "active"}},
                {"name": "% Idle Time", "metric": "windows.processor.time",
                 "attributes": {"state": "idle"}},
                {"name": "% User Time", "metric": "windows.processor.time",
                 "attributes": {"state": "user"}}
            ]},
            {"object": "Process", "instances": "worker#01", "counters": [
                {"name": "Private Bytes", "metric": "windows.process.private"}
            ]}
        ]
    }))
    .unwrap();
    let records = into_otap(
        &config.counters,
        sample(
            0,
            10,
            vec![
                point(3, SampleValue::NoObservation),
                point(1, SampleValue::Value(Number::Double(87.5))),
                point(2, SampleValue::NoObservation),
                point(0, SampleValue::Value(Number::Double(12.5))),
            ],
        ),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        records
            .get(ArrowPayloadType::UnivariateMetrics)
            .unwrap()
            .num_rows(),
        1
    );
    assert_eq!(
        records
            .get(ArrowPayloadType::NumberDataPoints)
            .unwrap()
            .num_rows(),
        2
    );
    let decoded = decode(records);
    let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].name, "windows.processor.time");
    let Some(metric::Data::Gauge(gauge)) = &metrics[0].data else {
        panic!("expected a Gauge");
    };
    assert_eq!(gauge.data_points.len(), 2);
    for (point, state, path, value) in [
        (
            &gauge.data_points[0],
            "idle",
            r"\Processor(_Total)\% Idle Time",
            87.5,
        ),
        (
            &gauge.data_points[1],
            "active",
            r"\Processor(_Total)\% Processor Time",
            12.5,
        ),
    ] {
        assert_eq!(
            attributes(&point.attributes),
            BTreeMap::from([("state", state), ("windows.perf_counter.path", path)])
        );
        assert_eq!(point.value, Some(number_data_point::Value::AsDouble(value)));
    }
}

/// Scenario: Explicit mappings use space-distinct, whitespace-only, and control-character static keys.
/// Guarantees: Decoded points retain every key/value exactly once, including static instance and padded near-reserved keys.
#[test]
fn projects_static_attribute_spelling_verbatim() {
    let expected = BTreeMap::from([
        ("state", "active"),
        ("State", "case-distinct"),
        (" state", " padded "),
        ("state ", ""),
        (" state ", "both"),
        (" \t\r\n\u{2003}", "whitespace"),
        ("control\0\t\n\u{7f}key", "control\0\t\n\u{7f}value"),
        (" windows.perf_counter.path", "not the native path"),
        ("Windows.perf_counter.path", "case-distinct prefix"),
        ("windows.perf_counter", "no trailing dot"),
        ("instance", "fixed"),
        (" instance ", "not an override"),
    ]);
    let config = RuntimeConfig::from_json(&serde_json::json!({
        "metrics": {"process.id": {"unit": "1"}},
        "perfcounters": [{
            "object": "Process",
            "instances": ["app", "worker"],
            "counters": [{
                "name": "ID Process",
                "metric": "process.id",
                "attributes": expected
            }]
        }]
    }))
    .unwrap();
    let decoded = decode(
        into_otap(
            &config.counters,
            sample(
                0,
                10,
                vec![
                    point(0, SampleValue::Value(Number::Integer(42))),
                    point(1, SampleValue::Value(Number::Integer(43))),
                ],
            ),
        )
        .unwrap()
        .unwrap(),
    );
    let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].name, "process.id");
    let Some(metric::Data::Gauge(gauge)) = &metrics[0].data else {
        panic!("expected a Gauge");
    };
    assert_eq!(gauge.data_points.len(), 2);
    for (index, point) in gauge.data_points.iter().enumerate() {
        let mut expected = expected.clone();
        let _ = expected.insert(
            "windows.perf_counter.path",
            config.counters[index].path.as_str(),
        );
        assert_eq!(point.attributes.len(), expected.len());
        assert_eq!(attributes(&point.attributes), expected);
    }
}

/// Scenario: Referenced Gauge and Sum definitions omit metadata while other definitions have no observations.
/// Guarantees: Decoded output retains default metadata and values, with no empty records for unused or omitted metrics.
#[test]
fn projects_optional_metric_metadata_without_empty_records() {
    let config = RuntimeConfig::from_json(&serde_json::json!({
        "metrics": {
            "gauge": {},
            "sum": {"unit": "", "sum": {"aggregation": "cumulative"}},
            "unobserved": {},
            "unused": {"sum": {"aggregation": "cumulative"}}
        },
        "perfcounters": [{
            "object": "Memory",
            "counters": [
                {"name": "Available Bytes", "metric": "gauge", "attributes": {"state": "free"}},
                {"name": "Committed Bytes", "metric": "sum", "attributes": {"state": "used"}},
                {"name": "Cache Bytes", "metric": "unobserved"}
            ]
        }]
    }))
    .unwrap();
    let decoded = decode(
        into_otap(
            &config.counters,
            sample(
                5,
                10,
                vec![
                    point(0, SampleValue::Value(Number::Integer(42))),
                    point(1, SampleValue::Value(Number::Double(12.5))),
                    point(2, SampleValue::NoObservation),
                ],
            ),
        )
        .unwrap()
        .unwrap(),
    );
    let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), 2);
    for (metric, name, path, state, value) in [
        (
            &metrics[0],
            "gauge",
            r"\Memory\Available Bytes",
            "free",
            number_data_point::Value::AsInt(42),
        ),
        (
            &metrics[1],
            "sum",
            r"\Memory\Committed Bytes",
            "used",
            number_data_point::Value::AsDouble(12.5),
        ),
    ] {
        assert_eq!(metric.name, name);
        assert_eq!(metric.unit, "1");
        assert!(metric.description.is_empty());
        let points = match metric.data.as_ref().unwrap() {
            metric::Data::Gauge(gauge) if name == "gauge" => &gauge.data_points,
            metric::Data::Sum(sum) if name == "sum" => {
                assert_eq!(
                    sum.aggregation_temporality,
                    AggregationTemporality::Cumulative as i32
                );
                assert!(!sum.is_monotonic);
                &sum.data_points
            }
            _ => panic!("unexpected metric kind"),
        };
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, Some(value));
        assert_eq!(points[0].time_unix_nano, 10);
        assert_eq!(
            points[0].start_time_unix_nano,
            if name == "gauge" { 0 } else { 5 }
        );
        assert_eq!(
            attributes(&points[0].attributes),
            BTreeMap::from([("state", state), ("windows.perf_counter.path", path),])
        );
    }
    assert!(
        into_otap(
            &config.counters,
            sample(
                5,
                10,
                (0..config.counters.len())
                    .map(|index| point(index, SampleValue::NoObservation))
                    .collect()
            ),
        )
        .unwrap()
        .is_none()
    );
}

/// Scenario: Omitted or empty references and an explicit reference resolve to an indexed path-named Gauge or Sum.
/// Guarantees: Configured index spelling selects the definition and reaches decoded paths, with explicit attributes only.
#[test]
fn projects_shared_path_definition_with_implicit_and_explicit_mappings() {
    let name = r"\Process(app#001)\Private Bytes";
    for (kind, definition) in [
        (MetricKind::Gauge, serde_json::json!({"gauge": {}})),
        (
            MetricKind::Sum,
            serde_json::json!({"sum": {"aggregation": "cumulative"}}),
        ),
    ] {
        let mut definition = definition;
        definition["unit"] = serde_json::json!("By");
        definition["description"] = serde_json::json!("Shared memory observations.");
        let mut value = serde_json::json!({
            "metrics": {name: definition, "unused": {}},
            "perfcounters": [{"object": "Process", "instances": "app#001", "counters": [
                {"name": "Private Bytes", "attributes": {"state": "ignored", "windows.perf_counter.path": "ignored"}},
                {"name": "Working Set", "metric": name, "attributes": {"state": "used"}}
            ]}]
        });
        for reference in [None, Some("")] {
            if let Some(reference) = reference {
                value["perfcounters"][0]["counters"][0]["metric"] = serde_json::json!(reference);
            }
            let config = RuntimeConfig::from_json(&value).unwrap();
            let decoded = decode(
                into_otap(
                    &config.counters,
                    sample(
                        5,
                        10,
                        vec![
                            point(1, SampleValue::Value(Number::Double(12.5))),
                            point(0, SampleValue::Value(Number::Integer(42))),
                        ],
                    ),
                )
                .unwrap()
                .unwrap(),
            );
            let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
            assert_eq!(metrics.len(), 1);
            let metric = &metrics[0];
            assert_eq!(metric.name, name);
            assert_eq!(metric.unit, "By");
            assert_eq!(metric.description, "Shared memory observations.");
            let points = match (kind, metric.data.as_ref().unwrap()) {
                (MetricKind::Gauge, metric::Data::Gauge(gauge)) => &gauge.data_points,
                (MetricKind::Sum, metric::Data::Sum(sum)) => {
                    assert_eq!(
                        sum.aggregation_temporality,
                        AggregationTemporality::Cumulative as i32
                    );
                    assert!(!sum.is_monotonic);
                    &sum.data_points
                }
                _ => panic!("unexpected metric kind"),
            };
            assert_eq!(points.len(), 2);
            assert_eq!(
                points[0].value,
                Some(number_data_point::Value::AsDouble(12.5))
            );
            assert_eq!(points[1].value, Some(number_data_point::Value::AsInt(42)));
            assert_eq!(
                attributes(&points[0].attributes),
                BTreeMap::from([
                    ("state", "used"),
                    (
                        "windows.perf_counter.path",
                        r"\Process(app#001)\Working Set"
                    ),
                ])
            );
            assert_eq!(
                attributes(&points[1].attributes),
                BTreeMap::from([(
                    "windows.perf_counter.path",
                    r"\Process(app#001)\Private Bytes"
                ),])
            );
            for point in points {
                assert_eq!(point.time_unix_nano, 10);
                assert_eq!(
                    point.start_time_unix_nano,
                    if kind == MetricKind::Sum { 5 } else { 0 }
                );
            }
            assert!(
                into_otap(
                    &config.counters,
                    sample(
                        5,
                        10,
                        vec![
                            point(0, SampleValue::NoObservation),
                            point(1, SampleValue::NoObservation)
                        ]
                    ),
                )
                .unwrap()
                .is_none()
            );
        }
    }
}

/// Scenario: Unreferenced paths differ from definitions by casing or numeric instance-index spelling.
/// Guarantees: Exact lookup falls back to path-named default Gauges and ignores attributes without losing points.
#[test]
fn projects_configured_path_names_with_exact_definition_lookup() {
    let mut value = serde_json::json!({
        "metrics": {
            r"\Memory\Available Bytes": {"unit": "By", "sum": {"aggregation": "cumulative"}},
            r"\Process(app)\Private Bytes": {"description": "Must not match", "sum": {"aggregation": "cumulative"}},
            r"\Process(worker#1)\Private Bytes": {"description": "Must not match", "sum": {"aggregation": "cumulative"}}
        },
        "perfcounters": [
            {"object": "memory", "counters": [{"name": "available bytes", "attributes": {"state": "ignored"}}]},
            {"object": "Process", "instances": ["app#00", "worker#001"], "counters": [
                {"name": "Private Bytes", "attributes": {"": "ignored", " ": "ignored", "instance": "ignored", "windows.perf_counter.path": "ignored"}}
            ]}
        ]
    });
    let definitions = value["metrics"].clone();
    for (definitions, reference) in [
        (None, None),
        (Some(serde_json::json!({})), Some("")),
        (Some(definitions), Some("")),
    ] {
        match definitions {
            Some(definitions) => value["metrics"] = definitions,
            None => {
                let _ = value.as_object_mut().unwrap().remove("metrics");
            }
        }
        if let Some(reference) = reference {
            for object in value["perfcounters"].as_array_mut().unwrap() {
                object["counters"][0]["metric"] = serde_json::json!(reference);
            }
        }
        let config = RuntimeConfig::from_json(&value).unwrap();
        let decoded = decode(
            into_otap(
                &config.counters,
                sample(
                    0,
                    10,
                    vec![
                        point(0, SampleValue::Value(Number::Integer(42))),
                        point(1, SampleValue::Value(Number::Double(12.5))),
                        point(2, SampleValue::Value(Number::Integer(-7))),
                    ],
                ),
            )
            .unwrap()
            .unwrap(),
        );
        let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(metrics.len(), 3);
        for (metric, expected_path, value) in [
            (
                &metrics[0],
                r"\memory\available bytes",
                number_data_point::Value::AsInt(42),
            ),
            (
                &metrics[1],
                r"\Process(app#00)\Private Bytes",
                number_data_point::Value::AsDouble(12.5),
            ),
            (
                &metrics[2],
                r"\Process(worker#001)\Private Bytes",
                number_data_point::Value::AsInt(-7),
            ),
        ] {
            assert_eq!(metric.name, expected_path);
            assert_eq!(metric.unit, "1");
            assert!(metric.description.is_empty());
            let Some(metric::Data::Gauge(gauge)) = &metric.data else {
                panic!("expected a fallback Gauge");
            };
            assert_eq!(gauge.data_points.len(), 1);
            let point = &gauge.data_points[0];
            assert_eq!(point.value, Some(value));
            assert_eq!(point.time_unix_nano, 10);
            assert_eq!(point.start_time_unix_nano, 0);
            assert_eq!(
                attributes(&point.attributes),
                BTreeMap::from([("windows.perf_counter.path", expected_path)])
            );
        }
    }
}

/// Scenario: A cumulative non-monotonic Sum has integer and scaled-double points from two instances.
/// Guarantees: Decoded output preserves Sum semantics, metadata, values, attributes and timestamps.
#[test]
fn sum_configuration_preserves_decoded_output() {
    let config = RuntimeConfig::from_json(&serde_json::json!({
        "metrics": {"windows.process.private": {
            "description": "Private bytes", "unit": "By",
            "sum": {"aggregation": "cumulative", "monotonic": false}
        }},
        "perfcounters": [{
            "object": "Process", "instances": ["app", "worker#01"],
            "counters": [{
                "name": "Private Bytes", "metric": "windows.process.private",
                "attributes": {"state": "used"}
            }]
        }]
    }))
    .unwrap();
    let start = 1_788_400_000_000_000_000;
    let timestamp = start + 123_456_789;
    let decoded = decode(
        into_otap(
            &config.counters,
            sample(
                start,
                timestamp,
                vec![
                    point(0, SampleValue::Value(Number::Integer(-42))),
                    point(1, SampleValue::Value(scale_integer(42, 3).unwrap())),
                ],
            ),
        )
        .unwrap()
        .unwrap(),
    );
    let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), 1);
    let metric = &metrics[0];
    assert_eq!(metric.name, "windows.process.private");
    assert_eq!(metric.description, "Private bytes");
    assert_eq!(metric.unit, "By");
    let Some(metric::Data::Sum(sum)) = &metric.data else {
        panic!("expected a Sum");
    };
    assert_eq!(
        sum.aggregation_temporality,
        AggregationTemporality::Cumulative as i32
    );
    assert!(!sum.is_monotonic);
    assert_eq!(sum.data_points.len(), 2);
    for (point, path, value) in [
        (
            &sum.data_points[0],
            r"\Process(app)\Private Bytes",
            number_data_point::Value::AsInt(-42),
        ),
        (
            &sum.data_points[1],
            r"\Process(worker#01)\Private Bytes",
            number_data_point::Value::AsDouble(42_000.0),
        ),
    ] {
        assert_eq!(point.value, Some(value));
        assert_eq!(point.start_time_unix_nano, start as u64);
        assert_eq!(point.time_unix_nano, timestamp as u64);
        assert_eq!(point.flags, 0);
        assert_eq!(
            attributes(&point.attributes),
            BTreeMap::from([("state", "used"), ("windows.perf_counter.path", path),])
        );
    }
}

/// Scenario: A Gauge carries zero, negative or future cumulative start-time state.
/// Guarantees: Projection ignores that state and never emits a Gauge start time.
#[test]
fn gauges_ignore_cumulative_start_time() {
    let counters = [counter("gauge", MetricKind::Gauge)];
    for start in [i64::MIN, -1, 0, 11, i64::MAX] {
        let records = into_otap(
            &counters,
            sample(
                start,
                10,
                vec![point(0, SampleValue::Value(Number::Integer(7)))],
            ),
        )
        .unwrap()
        .unwrap();
        let points = records.get(ArrowPayloadType::NumberDataPoints).unwrap();
        assert!(
            points
                .column_by_name("start_time_unix_nano")
                .unwrap()
                .is_null(0)
        );
    }
}

/// Scenario: An emitted cumulative Sum carries an invalid or boundary start timestamp.
/// Guarantees: Only 0 < start <= observation time is accepted; errors identify the counter and both times.
#[test]
fn validates_emitted_sum_start_time() {
    let counters = [counter("sum", MetricKind::Sum)];
    for start in [i64::MIN, -1, 0, 11, i64::MAX] {
        assert!(matches!(
            into_otap(
                &counters,
                sample(
                    start,
                    10,
                    vec![point(0, SampleValue::Value(Number::Integer(1)))]
                )
            ),
            Err(ArrowError::InvalidArgumentError(message))
                if message == format!(
                    "Sum sum (\\Object\\sum) has invalid start time {start} for timestamp 10"
                )
        ));
    }
    for start in [1, 10] {
        let decoded = decode(
            into_otap(
                &counters,
                sample(
                    start,
                    10,
                    vec![point(0, SampleValue::Value(Number::Integer(1)))],
                ),
            )
            .unwrap()
            .unwrap(),
        );
        let Some(metric::Data::Sum(sum)) =
            &decoded.resource_metrics[0].scope_metrics[0].metrics[0].data
        else {
            panic!("expected a Sum");
        };
        assert_eq!(sum.data_points[0].start_time_unix_nano, start as u64);
    }
}

/// Scenario: Observation time is zero or negative, with ready, omitted or empty points.
/// Guarantees: All samples require a positive observation timestamp; errors retain the invalid value.
#[test]
fn rejects_nonpositive_observation_time() {
    let counters = [counter("gauge", MetricKind::Gauge)];
    for timestamp in [i64::MIN, -1, 0] {
        for points in [
            vec![],
            vec![point(0, SampleValue::NoObservation)],
            vec![point(0, SampleValue::Value(Number::Integer(1)))],
        ] {
            assert!(matches!(
                into_otap(&counters, sample(1, timestamp, points)),
                Err(ArrowError::InvalidArgumentError(message))
                    if message == format!("sample has non-positive timestamp {timestamp}")
            ));
        }
    }
}

/// Scenario: A collection has no points or only no-observation points, including warming Sums.
/// Guarantees: No batch is emitted and unused cumulative start time is not validated.
#[test]
fn omits_empty_and_unobserved_samples() {
    let counters = [counter("sum", MetricKind::Sum)];
    for start in [-1, 0, 11] {
        for points in [vec![], vec![point(0, SampleValue::NoObservation)]] {
            assert!(
                into_otap(&counters, sample(start, 10, points))
                    .unwrap()
                    .is_none()
            );
        }
    }
    assert!(into_otap(&[], sample(0, 1, vec![])).unwrap().is_none());
}

/// Scenario: A point references a missing counter or an emit-ready value is a non-finite double.
/// Guarantees: Invalid indices are rejected even for omissions; non-finite errors retain the path and value.
#[test]
fn rejects_invalid_counter_references_and_values() {
    let counters = [counter("gauge", MetricKind::Gauge)];
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            into_otap(
                &counters,
                sample(
                    0,
                    1,
                    vec![point(0, SampleValue::Value(Number::Double(value)))]
                )
            ),
            Err(ArrowError::InvalidArgumentError(message))
                if message == format!("counter \\Object\\gauge has non-finite double value {value}")
        ));
    }
    for index in [1, usize::MAX] {
        for value in [
            SampleValue::Value(Number::Integer(1)),
            SampleValue::NoObservation,
        ] {
            assert!(matches!(
                into_otap(&counters, sample(0, 1, vec![point(index, value)])),
                Err(ArrowError::InvalidArgumentError(message))
                    if message == format!("sample references missing configured counter {index}")
            ));
        }
    }
}

/// Scenario: A configured counter occurs twice with numeric values, omissions or a mix of both.
/// Guarantees: A repeated counter rejects the whole sample, whether its points have values or not.
#[test]
fn rejects_duplicate_counter_indices() {
    let counters = [counter("gauge", MetricKind::Gauge)];
    for first in [
        SampleValue::Value(Number::Integer(7)),
        SampleValue::NoObservation,
    ] {
        for second in [
            SampleValue::Value(Number::Integer(9)),
            SampleValue::NoObservation,
        ] {
            assert!(matches!(
                into_otap(
                    &counters,
                    sample(0, 1, vec![point(0, first), point(0, second)])
                ),
                Err(ArrowError::InvalidArgumentError(message))
                    if message == r"sample repeats counter \Object\gauge (metric gauge) at index 0"
            ));
        }
    }
}
