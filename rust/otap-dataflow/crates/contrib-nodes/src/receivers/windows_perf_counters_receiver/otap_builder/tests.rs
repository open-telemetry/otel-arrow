// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::config::{Config, InstanceConfig};
use super::super::model::{InstanceIdentity, PendingMetadata, SampleFailure, SampleMember};
use super::super::test_utils::decode;
use super::*;
use arrow::array::{Array, Int64Array, UInt8Array};
use arrow::util::display::array_value_to_string;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric;

fn counter(path: &str, name: &str, unit: &str) -> CounterConfig {
    CounterConfig {
        path: path.to_owned(),
        name: Arc::from(name),
        unit: Arc::from(unit),
        description: Arc::from(format!("Description for {name}.")),
        metric_kind: MetricKind::Gauge,
        attributes: Arc::new(BTreeMap::new()),
        instance_config: InstanceConfig::None,
        scale_power10: 0,
        source_mapping_index: 0,
    }
}

fn non_monotonic_sum(path: &str, name: &str, unit: &str) -> CounterConfig {
    CounterConfig {
        metric_kind: MetricKind::Sum,
        ..counter(path, name, unit)
    }
}

fn point(counter_index: usize, path: &str, value: SampleValue) -> SamplePoint {
    point_at(counter_index, path, 1, value)
}

fn point_at(
    counter_index: usize,
    path: &str,
    timestamp_unix_nano: i64,
    value: SampleValue,
) -> SamplePoint {
    SamplePoint {
        counter_index,
        path: Arc::from(path),
        instance: None,
        emit_instance_attribute: false,
        timestamp_unix_nano,
        start_time_unix_nano: None,
        value,
    }
}

fn string_attributes(records: &OtapArrowRecords) -> Vec<(String, String, String)> {
    let attrs = records.get(ArrowPayloadType::NumberDpAttrs).unwrap();
    let parent_ids = attrs.column_by_name("parent_id").unwrap();
    let keys = attrs.column_by_name("key").unwrap();
    let values = attrs.column_by_name("str").unwrap();
    (0..attrs.num_rows())
        .map(|index| {
            (
                array_value_to_string(parent_ids.as_ref(), index).unwrap(),
                array_value_to_string(keys.as_ref(), index).unwrap(),
                array_value_to_string(values.as_ref(), index).unwrap(),
            )
        })
        .collect()
}

fn decoded_attributes(attributes: &[KeyValue]) -> BTreeMap<&str, &str> {
    attributes
        .iter()
        .map(|attribute| {
            let Some(any_value::Value::StringValue(value)) = attribute
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref())
            else {
                panic!("expected a string attribute");
            };
            (attribute.key.as_str(), value.as_str())
        })
        .collect()
}

fn sum_point(
    counter_index: usize,
    path: &str,
    start_time_unix_nano: i64,
    value: Number,
) -> SamplePoint {
    SamplePoint {
        timestamp_unix_nano: 10,
        start_time_unix_nano: Some(start_time_unix_nano),
        ..point(counter_index, path, SampleValue::Value(value))
    }
}

/// Scenario: An empty, omitted or valued sample has a non-positive envelope timestamp.
/// Guarantees: Every sample rejects the invalid timestamp and reports its exact value.
#[test]
fn rejects_nonpositive_sample_timestamps() {
    let path = r"\Memory\Available Bytes";
    let counters = [counter(path, "memory", "By")];
    for timestamp in [i64::MIN, -1, 0] {
        for points in [
            vec![],
            vec![point(0, path, SampleValue::NoObservation)],
            vec![point(0, path, SampleValue::Value(Number::Integer(42)))],
        ] {
            let error = into_otap(
                &counters,
                Sample {
                    timestamp_unix_nano: timestamp,
                    members: Vec::new(),
                    points,
                    metadata: Default::default(),
                },
            )
            .unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("Invalid argument error: sample has non-positive timestamp {timestamp}")
            );
        }
    }
}

/// Scenario: A later concrete Gauge has a non-positive timestamp, then becomes an omission.
/// Guarantees: The invalid value rejects the batch with exact path/time context; omitting it preserves the healthy peer.
#[test]
fn rejects_nonpositive_observation_timestamps() {
    let counters = [counter(r"\Process(*)\Private Bytes", "private", "By")];
    let path = r"\Process(worker)\Private Bytes";
    let healthy_path = r"\Process(healthy)\Private Bytes";
    for timestamp in [i64::MIN, -1, 0] {
        let mut sample = Sample {
            timestamp_unix_nano: 30,
            members: Vec::new(),
            points: vec![
                point_at(0, healthy_path, 10, SampleValue::Value(Number::Integer(1))),
                point_at(0, path, timestamp, SampleValue::Value(Number::Integer(2))),
            ],
            metadata: Default::default(),
        };
        let error = project_sample(&counters, &sample).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "Invalid argument error: counter {path} has non-positive timestamp {timestamp}"
            )
        );
        sample.points[1].value = SampleValue::NoObservation;
        let decoded = decode(into_otap(&counters, sample).unwrap().unwrap());
        let Some(metric::Data::Gauge(gauge)) =
            &decoded.resource_metrics[0].scope_metrics[0].metrics[0].data
        else {
            panic!("expected a Gauge");
        };
        assert_eq!(gauge.data_points.len(), 1);
        assert_eq!(gauge.data_points[0].time_unix_nano, 10);
        assert_eq!(
            decoded_attributes(&gauge.data_points[0].attributes)["windows.perf_counter.path"],
            healthy_path
        );
    }
}

/// Scenario: One mapping/path pair repeats with values or omissions and equal or differing times.
/// Guarantees: Neither omission nor observation time bypasses concrete-identity rejection or its exact diagnostic.
#[test]
fn rejects_duplicate_concrete_observations() {
    let path = r"\Process(worker)\Private Bytes";
    let counters = [counter(r"\Process(*)\Private Bytes", "private", "By")];
    for first in [
        SampleValue::Value(Number::Integer(7)),
        SampleValue::NoObservation,
    ] {
        for second in [
            SampleValue::Value(Number::Integer(8)),
            SampleValue::NoObservation,
        ] {
            for timestamp in [1, 2] {
                let error = into_otap(
                    &counters,
                    Sample {
                        timestamp_unix_nano: 2,
                        members: Vec::new(),
                        points: vec![point(0, path, first), point_at(0, path, timestamp, second)],
                        metadata: Default::default(),
                    },
                )
                .unwrap_err();
                assert_eq!(
                    error.to_string(),
                    format!(
                        "Invalid argument error: sample repeats counter {path} (metric private) at index 0"
                    )
                );
            }
        }
    }
}

/// Scenario: A concrete Gauge after a healthy peer contains NaN or either infinity.
/// Guarantees: Non-finite values reject the whole batch with the exact concrete path and value.
#[test]
fn rejects_nonfinite_values_as_invariants() {
    let path = r"\Process(worker)\Private Bytes";
    let counters = [counter(r"\Process(*)\Private Bytes", "private", "By")];
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = into_otap(
            &counters,
            Sample {
                timestamp_unix_nano: 30,
                members: Vec::new(),
                points: vec![
                    point_at(
                        0,
                        r"\Process(healthy)\Private Bytes",
                        10,
                        SampleValue::Value(Number::Integer(1)),
                    ),
                    point_at(0, path, 20, SampleValue::Value(Number::Double(value))),
                ],
                metadata: Default::default(),
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("Invalid argument error: counter {path} has non-finite double value {value}")
        );
    }
}

/// Scenario: A Gauge or Sum mapping carries two concrete paths with independent observation times.
/// Guarantees: Decoded points preserve concrete identity, times, values, shared metadata and static attributes, not the envelope time.
#[test]
fn projects_distinct_observations_under_one_mapping() {
    for kind in [MetricKind::Gauge, MetricKind::Sum] {
        let template = r"\Process(worker*)\Private Bytes";
        let mut configured = counter(template, "private.bytes", "By");
        configured.metric_kind = kind;
        configured.attributes = Arc::new(BTreeMap::from([("source".into(), "configured".into())]));
        let paths = [
            r"\Process(worker)\Private Bytes",
            r"\Process(worker#1)\Private Bytes",
        ];
        let points = paths
            .iter()
            .enumerate()
            .map(|(index, path)| SamplePoint {
                start_time_unix_nano: (kind == MetricKind::Sum).then_some(5),
                ..point_at(
                    0,
                    path,
                    10 + index as i64,
                    SampleValue::Value(Number::Integer(42)),
                )
            })
            .collect();
        let records = into_otap(
            &[configured],
            Sample {
                timestamp_unix_nano: 100,
                members: Vec::new(),
                points,
                metadata: Default::default(),
            },
        )
        .unwrap()
        .unwrap();
        let decoded = decode(records);
        let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "private.bytes");
        assert_eq!(metrics[0].unit, "By");
        let points = match metrics[0].data.as_ref().unwrap() {
            metric::Data::Gauge(gauge) => &gauge.data_points,
            metric::Data::Sum(sum) => {
                assert!(!sum.is_monotonic);
                assert_eq!(
                    sum.aggregation_temporality,
                    AGGREGATION_TEMPORALITY_CUMULATIVE
                );
                &sum.data_points
            }
            other => panic!("unexpected metric {other:?}"),
        };
        assert_eq!(points.len(), 2);
        for (index, point) in points.iter().enumerate() {
            assert_eq!(point.time_unix_nano, 10 + index as u64);
            assert_eq!(
                point.start_time_unix_nano,
                if kind == MetricKind::Sum { 5 } else { 0 }
            );
            let attributes = decoded_attributes(&point.attributes);
            assert_eq!(attributes["windows.perf_counter.path"], paths[index]);
            assert_eq!(attributes["windows.perf_counter.path_template"], template);
            assert_eq!(attributes["source"], "configured");
        }
    }
}

/// Scenario: An implicit path-named mapping matches a configured metric definition.
/// Guarantees: Decoded OTAP output uses the matching definition's name, description, unit, and kind.
#[test]
fn projects_implicit_path_named_metric_definition() {
    let config = Config::from_json(&serde_json::json!({
        "metrics": {
            r"\Memory\Available Bytes": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }
        },
        "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available Bytes"}]
        }]
    }))
    .unwrap();
    let records = into_otap(
        &config.counters,
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![point(
                0,
                r"\Memory\Available Bytes",
                SampleValue::Value(Number::Integer(42)),
            )],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();

    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    assert_eq!(metrics.num_rows(), 1);
    assert_eq!(
        metrics
            .column_by_name("metric_type")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap()
            .value(0),
        MetricType::Gauge as u8
    );
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(metrics))
        .unwrap()
        .to_string();
    assert!(display.contains(r"\Memory\Available Bytes"));
    assert!(display.contains("Available physical memory."));
    assert!(display.contains("By"));
}

/// Scenario: An implicit path metric uses a configured instance index with leading zeros.
/// Guarantees: OTAP uses the configured metric name and preserves the same native path attribute spelling.
#[test]
fn projects_configured_instance_spelling_for_path_metric() {
    let configured_name = r"\Process(app#001)\Private Bytes";
    let native_path = configured_name;
    let config = Config::from_json(&serde_json::json!({
        "metrics": {
            configured_name: {
                "description": "Configured process memory.",
                "unit": "By",
                "gauge": {}
            }
        },
        "perfcounters": [{
            "object": "Process",
            "instances": "app#001",
            "counters": [{"name": "Private Bytes"}]
        }]
    }))
    .unwrap();
    let records = into_otap(
        &config.counters,
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![point(
                0,
                native_path,
                SampleValue::Value(Number::Integer(42)),
            )],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();

    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(metrics))
        .unwrap()
        .to_string();
    assert!(display.contains(configured_name));
    assert!(display.contains("Configured process memory."));
    assert!(display.contains("By"));
    assert!(
        string_attributes(&records)
            .iter()
            .any(|(_, key, value)| key == "windows.perf_counter.path" && value == native_path)
    );
}

/// Scenario: Explicit mappings use space-distinct, whitespace-only, control-character, and near-reserved static keys.
/// Guarantees: Decoded points preserve exact spelling and values, and a static instance attribute takes precedence over generated instance identity.
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
    let config = Config::from_json(&serde_json::json!({
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
    let points = ["app", "worker"]
        .into_iter()
        .enumerate()
        .map(|(index, instance)| SamplePoint {
            counter_index: index,
            path: Arc::from(config.counters[index].path.as_str()),
            instance: Some(Arc::new(InstanceIdentity {
                name: instance.to_owned(),
                parent: None,
                index: 0,
            })),
            emit_instance_attribute: true,
            timestamp_unix_nano: 10,
            start_time_unix_nano: None,
            value: SampleValue::Value(Number::Integer(42 + index as i64)),
        })
        .collect();
    let decoded = decode(
        into_otap(
            &config.counters,
            Sample {
                timestamp_unix_nano: 10,
                members: Vec::new(),
                points,
                metadata: Default::default(),
            },
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
        let _ = expected.insert(
            "windows.perf_counter.instance",
            if index == 0 { "app" } else { "worker" },
        );
        let _ = expected.insert("windows.perf_counter.instance_index", "0");
        assert_eq!(point.attributes.len(), expected.len());
        assert_eq!(decoded_attributes(&point.attributes), expected);
    }
}

/// Scenario: An implicit configured path differs from a path-shaped definition only by case.
/// Guarantees: Exact lookup emits a default path-named Gauge instead of applying the mismatched definition.
#[test]
fn projects_case_distinct_path_name_fallback() {
    let configured_path = r"\memory\available bytes";
    let config = Config::from_json(&serde_json::json!({
        "metrics": {
            r"\Memory\Available Bytes": {
                "description": "Must not match.",
                "unit": "By",
                "sum": {"aggregation": "cumulative"}
            }
        },
        "perfcounters": [{
            "object": "memory",
            "counters": [{"name": "available bytes"}]
        }]
    }))
    .unwrap();
    let records = into_otap(
        &config.counters,
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![point(
                0,
                configured_path,
                SampleValue::Value(Number::Integer(42)),
            )],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();

    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    assert_eq!(
        metrics
            .column_by_name("metric_type")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap()
            .value(0),
        MetricType::Gauge as u8
    );
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(metrics))
        .unwrap()
        .to_string();
    assert!(display.contains(configured_path));
    assert!(!display.contains("Must not match."));
    assert!(
        string_attributes(&records).iter().any(|(_, key, value)| {
            key == "windows.perf_counter.path" && value == configured_path
        })
    );
}

/// Scenario: Multiple configured readings include an integer beyond f64's exact range.
/// Guarantees: Gauge order, metadata, paths, timestamps, and exact i64 values are retained.
#[test]
fn projects_integer_gauges() {
    let timestamp = 1_788_500_000_123_456_789;
    let bytes = 9_007_199_254_740_993;
    let counters = [
        counter(r"\Memory\Available Bytes", "windows.memory.available", "By"),
        counter(r"\Memory\Committed Bytes", "windows.memory.committed", "By"),
    ];
    let records = into_otap(
        &counters,
        Sample {
            timestamp_unix_nano: timestamp,
            members: Vec::new(),
            points: vec![
                point_at(
                    0,
                    r"\Memory\Available Bytes",
                    timestamp,
                    SampleValue::Value(Number::Integer(bytes)),
                ),
                point_at(
                    1,
                    r"\Memory\Committed Bytes",
                    timestamp,
                    SampleValue::Value(Number::Integer(42)),
                ),
            ],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();
    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    let points = records.get(ArrowPayloadType::NumberDataPoints).unwrap();
    assert_eq!(metrics.num_rows(), 2);
    assert_eq!(points.num_rows(), 2);
    assert_eq!(
        metrics
            .column_by_name("metric_type")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap()
            .value(0),
        MetricType::Gauge as u8
    );
    assert_eq!(
        points
            .column_by_name("int_value")
            .unwrap()
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        bytes
    );
    assert!(
        points
            .column_by_name("double_value")
            .is_none_or(|column| column.is_null(0))
    );
    assert!(
        points
            .column_by_name("start_time_unix_nano")
            .is_none_or(|column| column.is_null(0))
    );
    // Pretty-printing the record batches also exercises the dictionary-aware Arrow display.
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(metrics))
        .unwrap()
        .to_string();
    assert!(display.contains("windows.memory.available"));
    assert!(display.contains("windows.memory.committed"));
    assert!(display.contains("Description for windows.memory.available."));
    assert!(display.contains("By"));
    let attrs = records.get(ArrowPayloadType::NumberDpAttrs).unwrap();
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(attrs))
        .unwrap()
        .to_string();
    assert!(display.contains(r"\Memory\Available Bytes"));
    assert!(display.contains(r"\Memory\Committed Bytes"));
    assert!(display.contains("windows.perf_counter.path"));
    let time = points
        .column_by_name("time_unix_nano")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::TimestampNanosecondArray>()
        .unwrap();
    assert_eq!(time.value(0), timestamp);
}

/// Scenario: Process memory is configured as a non-monotonic Sum with a current integer value.
/// Guarantees: OTAP emits a cumulative non-monotonic Sum and preserves the query start time.
#[test]
fn projects_integer_non_monotonic_sum() {
    let mut counter = non_monotonic_sum(
        r"\Process(*)\Private Bytes",
        "windows.process.private",
        "By",
    );
    counter.description = Arc::from("Committed private memory for each process instance.");
    let counters = [counter];
    let records = into_otap(
        &counters,
        Sample {
            timestamp_unix_nano: 10,
            members: Vec::new(),
            points: vec![sum_point(
                0,
                r"\Process(worker)\Private Bytes",
                5,
                Number::Integer(42),
            )],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();
    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    assert_eq!(
        metrics
            .column_by_name("metric_type")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap()
            .value(0),
        MetricType::Sum as u8
    );
    assert_eq!(
        array_value_to_string(
            metrics
                .column_by_name("aggregation_temporality")
                .unwrap()
                .as_ref(),
            0
        )
        .unwrap(),
        AGGREGATION_TEMPORALITY_CUMULATIVE.to_string()
    );
    assert!(metrics.column_by_name("is_monotonic").is_none());
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(metrics))
        .unwrap()
        .to_string();
    assert!(display.contains("Committed private memory for each process instance."));
    let points = records.get(ArrowPayloadType::NumberDataPoints).unwrap();
    assert_eq!(
        points
            .column_by_name("start_time_unix_nano")
            .unwrap()
            .as_any()
            .downcast_ref::<arrow::array::TimestampNanosecondArray>()
            .unwrap()
            .value(0),
        5
    );
}

/// Scenario: A ready Sum point reaches projection without its required cumulative or delta interval start.
/// Guarantees: Projection reports an internal invariant error instead of emitting a Sum with an absent start time.
#[test]
fn rejects_sum_without_start_time() {
    let counters = [non_monotonic_sum(
        r"\Process(*)\Private Bytes",
        "windows.process.private",
        "By",
    )];
    let error = into_otap(
        &counters,
        Sample {
            timestamp_unix_nano: 10,
            members: Vec::new(),
            points: vec![point(
                0,
                r"\Process(worker)\Private Bytes",
                SampleValue::Value(Number::Integer(42)),
            )],
            metadata: Default::default(),
        },
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ArrowError::InvalidArgumentError(message)
            if message == "Sum windows.process.private (\\Process(worker)\\Private Bytes) has no start time"
    ));
}

/// Scenario: A cumulative Sum carries invalid starts or the accepted positive/equal-time boundaries.
/// Guarantees: Only 0 < start <= observation time emits; invalid starts retain both exact values in the diagnostic.
#[test]
fn validates_emitted_sum_start_time() {
    let path = r"\Memory\Available Bytes";
    let counters = [non_monotonic_sum(path, "memory", "By")];
    let sample = |start| Sample {
        timestamp_unix_nano: 30,
        members: Vec::new(),
        points: vec![sum_point(0, path, start, Number::Integer(42))],
        metadata: Default::default(),
    };
    for start in [i64::MIN, -1, 0, 11, i64::MAX] {
        let error = into_otap(&counters, sample(start)).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "Invalid argument error: Sum memory ({path}) has invalid start time {start} for timestamp 10"
            )
        );
    }
    for start in [1, 10] {
        let decoded = decode(into_otap(&counters, sample(start)).unwrap().unwrap());
        let Some(metric::Data::Sum(sum)) =
            &decoded.resource_metrics[0].scope_metrics[0].metrics[0].data
        else {
            panic!("expected a Sum");
        };
        assert_eq!(sum.data_points.len(), 1);
        assert_eq!(sum.data_points[0].start_time_unix_nano, start as u64);
        assert_eq!(sum.data_points[0].time_unix_nano, 10);
    }
}

/// Scenario: Samples are empty or contain only omitted Sums with missing or invalid start/time state.
/// Guarantees: Omissions require no output timestamps and never synthesize an empty batch.
#[test]
fn omits_empty_and_unobserved_samples() {
    let path = r"\Memory\Available Bytes";
    let counters = [non_monotonic_sum(path, "memory", "By")];
    assert!(
        into_otap(
            &[],
            Sample {
                timestamp_unix_nano: 30,
                members: Vec::new(),
                points: Vec::new(),
                metadata: Default::default(),
            },
        )
        .unwrap()
        .is_none()
    );
    for start in [
        None,
        Some(i64::MIN),
        Some(-1),
        Some(0),
        Some(11),
        Some(i64::MAX),
    ] {
        for timestamp in [i64::MIN, -1, 0, 10] {
            let omitted = SamplePoint {
                start_time_unix_nano: start,
                ..point_at(0, path, timestamp, SampleValue::NoObservation)
            };
            assert!(
                into_otap(
                    &counters,
                    Sample {
                        timestamp_unix_nano: 30,
                        members: Vec::new(),
                        points: vec![omitted],
                        metadata: Default::default(),
                    },
                )
                .unwrap()
                .is_none()
            );
        }
    }
}

/// Scenario: A Gauge carries negative, zero or future cumulative start-time state.
/// Guarantees: Every irrelevant start is ignored and the Gauge start column remains null.
#[test]
fn gauge_ignores_irrelevant_start_time() {
    let path = r"\Memory\Available Bytes";
    let counters = [counter(path, "windows.memory.available", "By")];
    for start in [i64::MIN, -1, 0, 11, i64::MAX] {
        let observation = SamplePoint {
            start_time_unix_nano: Some(start),
            ..point_at(0, path, 10, SampleValue::Value(Number::Integer(42)))
        };
        let records = into_otap(
            &counters,
            Sample {
                timestamp_unix_nano: 10,
                members: Vec::new(),
                points: vec![observation],
                metadata: Default::default(),
            },
        )
        .unwrap()
        .unwrap();
        let points = records.get(ArrowPayloadType::NumberDataPoints).unwrap();
        assert_eq!(points.num_rows(), 1);
        assert!(
            points
                .column_by_name("start_time_unix_nano")
                .unwrap()
                .is_null(0)
        );
    }
}

/// Scenario: A ready Sum with missing or future start follows a healthy concrete observation.
/// Guarantees: A healthy peer cannot hide a later Sum invariant failure or its path and times.
#[test]
fn rejects_invalid_sum_start_after_healthy_peer() {
    let path = r"\Process(worker)\Private Bytes";
    let counters = [non_monotonic_sum(
        r"\Process(*)\Private Bytes",
        "private",
        "By",
    )];
    for (start, expected) in [
        (None, format!("Sum private ({path}) has no start time")),
        (
            Some(21),
            format!("Sum private ({path}) has invalid start time 21 for timestamp 20"),
        ),
    ] {
        let error = into_otap(
            &counters,
            Sample {
                timestamp_unix_nano: 30,
                members: Vec::new(),
                points: vec![
                    sum_point(0, r"\Process(healthy)\Private Bytes", 5, Number::Integer(1)),
                    SamplePoint {
                        start_time_unix_nano: start,
                        ..point_at(0, path, 20, SampleValue::Value(Number::Integer(2)))
                    },
                ],
                metadata: Default::default(),
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("Invalid argument error: {expected}")
        );
    }
}

/// Scenario: The immediate first scrape has a ready direct gauge and a warming calculated gauge.
/// Guarantees: The first batch contains only the direct gauge, without delaying or zero-filling it.
#[test]
fn first_scrape_emits_ready_direct_gauge() {
    let counters = [
        counter(r"\Memory\Available Bytes", "windows.memory.available", "By"),
        counter(
            r"\Processor(_Total)\% Processor Time",
            "windows.processor.time",
            "%",
        ),
    ];
    let records = into_otap(
        &counters,
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![
                point(
                    0,
                    r"\Memory\Available Bytes",
                    SampleValue::Value(Number::Integer(42)),
                ),
                point(
                    1,
                    r"\Processor(_Total)\% Processor Time",
                    SampleValue::Warming,
                ),
            ],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();
    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(metrics))
        .unwrap()
        .to_string();
    assert_eq!(metrics.num_rows(), 1);
    assert!(display.contains("windows.memory.available"));
    assert!(!display.contains("windows.processor.time"));
}

/// Scenario: Every configured calculated counter is still warming.
/// Guarantees: The receiver can skip the scrape without constructing an empty metric batch.
#[test]
fn suppresses_all_warming_scrape() {
    let counters = [counter(
        r"\Processor(_Total)\% Processor Time",
        "windows.processor.time",
        "%",
    )];
    assert!(
        into_otap(
            &counters,
            Sample {
                timestamp_unix_nano: 1,
                members: Vec::new(),
                points: vec![point(
                    0,
                    r"\Processor(_Total)\% Processor Time",
                    SampleValue::Warming,
                )],
                metadata: Default::default(),
            }
        )
        .unwrap()
        .is_none()
    );
}

/// Scenario: An idle average has no observations while Memory and CPU have ready values.
/// Guarantees: Only the idle average is omitted and ready gauges remain in the batch.
#[test]
fn omits_idle_average_without_dropping_ready_values() {
    let counters = [
        counter(r"\Memory\Available Bytes", "windows.memory.available", "By"),
        counter(
            r"\Processor(_Total)\% Processor Time",
            "windows.processor.time",
            "%",
        ),
        counter(
            r"\PhysicalDisk(_Total)\Avg. Disk Bytes/Read",
            "windows.disk.read.average",
            "By",
        ),
    ];
    let records = into_otap(
        &counters,
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![
                point(
                    0,
                    r"\Memory\Available Bytes",
                    SampleValue::Value(Number::Integer(42)),
                ),
                point(
                    1,
                    r"\Processor(_Total)\% Processor Time",
                    SampleValue::Value(Number::Double(12.5)),
                ),
                point(
                    2,
                    r"\PhysicalDisk(_Total)\Avg. Disk Bytes/Read",
                    SampleValue::NoObservation,
                ),
            ],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();
    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(metrics))
        .unwrap()
        .to_string();
    assert_eq!(metrics.num_rows(), 2);
    assert!(display.contains("windows.memory.available"));
    assert!(display.contains("windows.processor.time"));
    assert!(!display.contains("windows.disk.read.average"));
}

/// Scenario: A disappeared wildcard instance fails while an exact Memory point remains valid.
/// Guarantees: Counter-local failure metadata does not suppress projection of healthy points.
#[test]
fn projects_healthy_points_when_a_peer_failed() {
    let counters = [
        counter(r"\Memory\Available Bytes", "windows.memory.available", "By"),
        counter(
            r"\Process(*)\Private Bytes",
            "windows.process.private",
            "By",
        ),
    ];
    let records = into_otap(
        &counters,
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![point(
                0,
                r"\Memory\Available Bytes",
                SampleValue::Value(Number::Integer(42)),
            )],
            metadata: PendingMetadata {
                failures: vec![SampleFailure {
                    counter_index: 1,
                    path: Arc::from(r"\Memory\Committed Bytes"),
                    reason: "PdhGetFormattedCounterValue",
                    error: "PDH_INVALID_DATA".to_owned(),
                    diagnostic_only: false,
                }],
                ..Default::default()
            },
        },
    )
    .unwrap()
    .unwrap();
    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(metrics))
        .unwrap()
        .to_string();
    assert_eq!(metrics.num_rows(), 1);
    assert!(display.contains("windows.memory.available"));
    assert!(!display.contains("windows.process.private"));
}

/// Scenario: A calculated counter produces a finite floating-point value.
/// Guarantees: OTAP uses the double column and does not synthesize an integer value.
#[test]
fn projects_calculated_double_gauge() {
    let counters = [counter(
        r"\Processor(_Total)\% Processor Time",
        "windows.processor.time",
        "%",
    )];
    let records = into_otap(
        &counters,
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![point(
                0,
                r"\Processor(_Total)\% Processor Time",
                SampleValue::Value(Number::Double(12.5)),
            )],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();
    let points = records.get(ArrowPayloadType::NumberDataPoints).unwrap();
    let doubles = points
        .column_by_name("double_value")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::Float64Array>()
        .unwrap();
    assert_eq!(doubles.value(0), 12.5);
    assert!(
        points
            .column_by_name("int_value")
            .is_none_or(|column| column.is_null(0))
    );
}

/// Scenario: A value or omission references a missing counter with or without admitted membership.
/// Guarantees: Invalid indices fail before omission or membership can hide them, with the exact index in the error.
#[test]
fn rejects_missing_counter_index_as_invariant() {
    let counters = [counter(
        r"\Memory\Available Bytes",
        "windows.memory.available",
        "By",
    )];
    for index in [1, usize::MAX] {
        for value in [
            SampleValue::NoObservation,
            SampleValue::Value(Number::Integer(42)),
        ] {
            for members in [
                Vec::new(),
                vec![SampleMember {
                    counter_index: 0,
                    path: Arc::from(r"\Memory\Available Bytes"),
                    instance: None,
                    emit_instance_attribute: false,
                }],
            ] {
                let error = project_sample(
                    &counters,
                    &Sample {
                        timestamp_unix_nano: 1,
                        members,
                        points: vec![point(index, r"\Memory\Available Bytes", value)],
                        metadata: Default::default(),
                    },
                )
                .unwrap_err();
                assert_eq!(
                    error.to_string(),
                    format!(
                        "Invalid argument error: sample references missing configured counter {index}"
                    )
                );
            }
        }
    }
}

/// Scenario: A point references a concrete source absent from the worker's admitted membership snapshot.
/// Guarantees: Corrupt membership cannot silently masquerade as a suppressed output collision.
#[test]
fn rejects_point_absent_from_admitted_membership() {
    let counters = [counter(
        r"\Process(*)\Private Bytes",
        "process.private",
        "By",
    )];
    let error = project_sample(
        &counters,
        &Sample {
            timestamp_unix_nano: 1,
            members: vec![SampleMember {
                counter_index: 0,
                path: Arc::from(r"\Process(owner)\Private Bytes"),
                instance: None,
                emit_instance_attribute: false,
            }],
            points: vec![point(
                0,
                r"\Process(untracked)\Private Bytes",
                SampleValue::Value(Number::Integer(42)),
            )],
            metadata: Default::default(),
        },
    )
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        r"Invalid argument error: sample point 0 (\Process(untracked)\Private Bytes) is absent from admitted membership"
    );
}

/// Scenario: Two returned instances map to one configured pure question-mark wildcard metric.
/// Guarantees: Output uses concrete returned names and duplicate identities rather than the configured pattern.
#[test]
fn projects_wildcard_instance_identity() {
    let mut wildcard = counter(
        r"\Process(svc?ost)\Private Bytes",
        "windows.process.private",
        "By",
    );
    wildcard.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let counters = [wildcard];
    let records = into_otap(
        &counters,
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![
                SamplePoint {
                    counter_index: 0,
                    path: Arc::from(r"\Process(svchost)\Private Bytes"),
                    instance: Some(Arc::new(InstanceIdentity {
                        name: "svchost".to_owned(),
                        parent: None,
                        index: 0,
                    })),
                    emit_instance_attribute: true,
                    timestamp_unix_nano: 1,
                    start_time_unix_nano: None,
                    value: SampleValue::Value(Number::Integer(10)),
                },
                SamplePoint {
                    counter_index: 0,
                    path: Arc::from(r"\Process(svchost#1)\Private Bytes"),
                    instance: Some(Arc::new(InstanceIdentity {
                        name: "svchost".to_owned(),
                        parent: Some("service".to_owned()),
                        index: 1,
                    })),
                    emit_instance_attribute: true,
                    timestamp_unix_nano: 1,
                    start_time_unix_nano: None,
                    value: SampleValue::Value(Number::Integer(20)),
                },
            ],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();
    let metrics = records.get(ArrowPayloadType::UnivariateMetrics).unwrap();
    assert_eq!(metrics.num_rows(), 1);
    let points = records.get(ArrowPayloadType::NumberDataPoints).unwrap();
    assert_eq!(points.num_rows(), 2);
    let attrs = records.get(ArrowPayloadType::NumberDpAttrs).unwrap();
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(attrs))
        .unwrap()
        .to_string();
    assert!(display.contains(r"\Process(svchost)\Private Bytes"));
    assert!(display.contains(r"\Process(svchost#1)\Private Bytes"));
    assert!(display.contains(r"\Process(svc?ost)\Private Bytes"));
    assert!(display.contains("windows.perf_counter.instance"));
    assert!(display.contains("windows.perf_counter.instance_index"));
    assert!(display.contains("windows.perf_counter.parent_instance"));
    assert!(display.contains("svchost"));
    assert!(display.contains("service"));
    assert_eq!(
        string_attributes(&records)
            .into_iter()
            .filter(|(_, key, _)| key == "instance")
            .collect::<Vec<_>>(),
        [
            ("0".to_owned(), "instance".to_owned(), "svchost".to_owned()),
            (
                "1".to_owned(),
                "instance".to_owned(),
                "svchost#1".to_owned()
            )
        ]
    );
}

/// Scenario: An aggregate is collected alone or explicitly included with other instances.
/// Guarantees: Go's `instance` attribute is omitted only for the aggregate-only mapping.
#[test]
fn projects_aggregate_instance_selection() {
    let mut aggregate_alone = counter(r"\Processor(_Total)\Counter", "aggregate.alone", "1");
    aggregate_alone.instance_config = InstanceConfig::Exact {
        emit_attribute: false,
    };
    let mut explicitly_included = counter(r"\Processor(*)\Counter", "aggregate.included", "1");
    explicitly_included.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let aggregate = Arc::new(InstanceIdentity {
        name: "_Total".to_owned(),
        parent: None,
        index: 0,
    });
    let records = into_otap(
        &[aggregate_alone, explicitly_included],
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![
                SamplePoint {
                    counter_index: 0,
                    path: Arc::from(r"\Processor(_Total)\Counter"),
                    instance: Some(aggregate.clone()),
                    emit_instance_attribute: false,
                    timestamp_unix_nano: 1,
                    start_time_unix_nano: None,
                    value: SampleValue::Value(Number::Integer(10)),
                },
                SamplePoint {
                    counter_index: 1,
                    path: Arc::from(r"\Processor(_Total)\Counter"),
                    instance: Some(aggregate),
                    emit_instance_attribute: true,
                    timestamp_unix_nano: 1,
                    start_time_unix_nano: None,
                    value: SampleValue::Value(Number::Integer(20)),
                },
            ],
            metadata: Default::default(),
        },
    )
    .unwrap()
    .unwrap();

    let attrs = string_attributes(&records);
    assert_eq!(
        attrs
            .iter()
            .filter(|(_, key, _)| key == "instance")
            .cloned()
            .collect::<Vec<_>>(),
        [("1".to_owned(), "instance".to_owned(), "_Total".to_owned())]
    );
    assert_eq!(
        attrs
            .iter()
            .filter(|(_, key, _)| key == "windows.perf_counter.instance")
            .count(),
        2
    );
}

/// Scenario: Two native instances in one mapping share a fixed user `instance` attribute.
/// Guarantees: Both decoded points retain the static override and distinct native path/instance provenance without suppression.
#[test]
fn static_instance_override_preserves_both_native_sources() {
    let mut wildcard = counter(r"\Process(*)\Private Bytes", "process.private", "By");
    wildcard.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    wildcard.attributes = Arc::new(BTreeMap::from([(
        "instance".to_owned(),
        "fixed".to_owned(),
    )]));
    let outcome = project_sample(
        &[wildcard],
        &Sample {
            timestamp_unix_nano: 1,
            members: ["worker", "service"]
                .into_iter()
                .map(|name| SampleMember {
                    counter_index: 0,
                    path: Arc::from(format!(r"\Process({name})\Private Bytes")),
                    instance: Some(Arc::new(InstanceIdentity {
                        name: name.to_owned(),
                        parent: None,
                        index: 0,
                    })),
                    emit_instance_attribute: true,
                })
                .collect(),
            points: vec![
                SamplePoint {
                    counter_index: 0,
                    path: Arc::from(r"\Process(worker)\Private Bytes"),
                    instance: Some(Arc::new(InstanceIdentity {
                        name: "worker".to_owned(),
                        parent: None,
                        index: 0,
                    })),
                    emit_instance_attribute: true,
                    timestamp_unix_nano: 1,
                    start_time_unix_nano: None,
                    value: SampleValue::Value(Number::Integer(10)),
                },
                SamplePoint {
                    counter_index: 0,
                    path: Arc::from(r"\Process(service)\Private Bytes"),
                    instance: Some(Arc::new(InstanceIdentity {
                        name: "service".to_owned(),
                        parent: None,
                        index: 0,
                    })),
                    emit_instance_attribute: true,
                    timestamp_unix_nano: 1,
                    start_time_unix_nano: None,
                    value: SampleValue::Value(Number::Integer(20)),
                },
            ],
            metadata: Default::default(),
        },
    )
    .unwrap();
    let request = decode(outcome.unwrap());
    let metric = &request.resource_metrics[0].scope_metrics[0].metrics[0];
    let Some(metric::Data::Gauge(gauge)) = &metric.data else {
        panic!("expected Gauge");
    };
    assert_eq!(gauge.data_points.len(), 2);
    for (point, name) in gauge.data_points.iter().zip(["worker", "service"]) {
        let attrs = decoded_attributes(&point.attributes);
        assert_eq!(attrs["instance"], "fixed");
        assert_eq!(attrs["windows.perf_counter.instance"], name);
        assert_eq!(
            attrs["windows.perf_counter.path"],
            format!(r"\Process({name})\Private Bytes")
        );
        assert_eq!(
            attrs["windows.perf_counter.path_template"],
            r"\Process(*)\Private Bytes"
        );
    }
}

/// Scenario: Distinct configured patterns overlap one concrete path while the first mapping initially warms.
/// Guarantees: Decoded output publishes each valid mapping independently and preserves both template-qualified points when ready.
#[test]
fn overlapping_patterns_publish_each_valid_mapping() {
    let concrete = r"\Process(svchost)\Private Bytes";
    let instance = Arc::new(InstanceIdentity {
        name: "svchost".to_owned(),
        parent: None,
        index: 0,
    });
    let mut first = counter(r"\Process(svc*)\Private Bytes", "process.private", "By");
    first.source_mapping_index = 0;
    first.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut second = counter(r"\Process(*host)\Private Bytes", "process.private", "By");
    second.source_mapping_index = 1;
    second.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let members = [0, 1]
        .into_iter()
        .map(|counter_index| SampleMember {
            counter_index,
            path: Arc::from(concrete),
            instance: Some(Arc::clone(&instance)),
            emit_instance_attribute: true,
        })
        .collect();
    let mut sample = Sample {
        timestamp_unix_nano: 1,
        members,
        points: vec![
            SamplePoint {
                counter_index: 0,
                path: Arc::from(concrete),
                instance: Some(Arc::clone(&instance)),
                emit_instance_attribute: true,
                timestamp_unix_nano: 1,
                start_time_unix_nano: None,
                value: SampleValue::Warming,
            },
            SamplePoint {
                counter_index: 1,
                path: Arc::from(concrete),
                instance: Some(instance),
                emit_instance_attribute: true,
                timestamp_unix_nano: 1,
                start_time_unix_nano: None,
                value: SampleValue::Value(Number::Integer(20)),
            },
        ],
        metadata: Default::default(),
    };

    let first_request = decode(
        project_sample(&[first.clone(), second.clone()], &sample)
            .unwrap()
            .unwrap(),
    );
    let Some(metric::Data::Gauge(gauge)) =
        &first_request.resource_metrics[0].scope_metrics[0].metrics[0].data
    else {
        panic!("expected Gauge");
    };
    assert_eq!(gauge.data_points.len(), 1);
    assert_eq!(
        decoded_attributes(&gauge.data_points[0].attributes)["windows.perf_counter.path_template"],
        second.path
    );

    sample.points[0].value = SampleValue::Value(Number::Integer(10));
    let recovered = project_sample(&[first, second], &sample).unwrap();
    let request = decode(recovered.unwrap());
    let Some(metric::Data::Gauge(gauge)) =
        &request.resource_metrics[0].scope_metrics[0].metrics[0].data
    else {
        panic!("expected Gauge");
    };
    assert_eq!(gauge.data_points.len(), 2);
    for (point, template) in gauge.data_points.iter().zip([
        r"\Process(svc*)\Private Bytes",
        r"\Process(*host)\Private Bytes",
    ]) {
        let attrs = decoded_attributes(&point.attributes);
        assert_eq!(attrs["windows.perf_counter.path_template"], template);
        assert_eq!(attrs["windows.perf_counter.path"], concrete);
        assert_eq!(attrs["instance"], "svchost");
    }
}

/// Scenario: Two different exact native paths feed one metric without static disambiguation.
/// Guarantees: Both decoded Gauge points survive under their retained concrete-path attributes.
#[test]
fn distinct_exact_paths_publish_both_points_of_shared_metric() {
    let config = Config::from_json(&serde_json::json!({
        "metrics": {"memory.value": {"unit": "By", "gauge": {}}},
        "perfcounters": [{"object": "Memory", "counters": [
            {"name": "Available Bytes", "metric": "memory.value"},
            {"name": "Committed Bytes", "metric": "memory.value"}
        ]}]
    }))
    .unwrap();
    let points = config
        .counters
        .iter()
        .enumerate()
        .map(|(index, counter)| {
            point(
                index,
                &counter.path,
                SampleValue::Value(Number::Integer(10 + index as i64)),
            )
        })
        .collect::<Vec<_>>();
    let members = points
        .iter()
        .map(|point| SampleMember {
            counter_index: point.counter_index,
            path: Arc::clone(&point.path),
            instance: None,
            emit_instance_attribute: false,
        })
        .collect();
    let request = decode(
        into_otap(
            &config.counters,
            Sample {
                timestamp_unix_nano: 1,
                members,
                points,
                metadata: Default::default(),
            },
        )
        .unwrap()
        .unwrap(),
    );
    let metrics = &request.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), 1);
    let Some(metric::Data::Gauge(gauge)) = &metrics[0].data else {
        panic!("expected Gauge");
    };
    assert_eq!(gauge.data_points.len(), 2);
    let paths = gauge
        .data_points
        .iter()
        .map(|point| decoded_attributes(&point.attributes)["windows.perf_counter.path"])
        .collect::<BTreeSet<_>>();
    assert_eq!(
        paths,
        BTreeSet::from([r"\Memory\Available Bytes", r"\Memory\Committed Bytes"])
    );
}

/// Scenario: Two counters map to one metric and use custom attributes to distinguish their points.
/// Guarantees: OTAP emits one metric row and preserves both attribute-qualified gauge points.
#[test]
fn projects_shared_metric_with_custom_attributes() {
    let mut active = counter(
        r"\Processor(_Total)\% Processor Time",
        "windows.processor.time",
        "%",
    );
    active.attributes = Arc::new(BTreeMap::from([("state".to_owned(), "active".to_owned())]));
    let mut idle = counter(
        r"\Processor(_Total)\% Idle Time",
        "windows.processor.time",
        "%",
    );
    idle.attributes = Arc::new(BTreeMap::from([("state".to_owned(), "idle".to_owned())]));
    let records = into_otap(
        &[active, idle],
        Sample {
            timestamp_unix_nano: 1,
            members: Vec::new(),
            points: vec![
                point(
                    0,
                    r"\Processor(_Total)\% Processor Time",
                    SampleValue::Value(Number::Double(60.0)),
                ),
                point(
                    1,
                    r"\Processor(_Total)\% Idle Time",
                    SampleValue::Value(Number::Double(40.0)),
                ),
            ],
            metadata: Default::default(),
        },
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
    let attrs = records.get(ArrowPayloadType::NumberDpAttrs).unwrap();
    let display = arrow::util::pretty::pretty_format_batches(std::slice::from_ref(attrs))
        .unwrap()
        .to_string();
    assert!(display.contains("state"));
    assert!(display.contains("active"));
    assert!(display.contains("idle"));
}
