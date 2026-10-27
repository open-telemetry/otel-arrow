// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::config::Config;
use super::super::pdh::Point;
use super::super::test_utils::decode;
use super::*;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    Metric, NumberDataPoint, metric, number_data_point,
};
use serde_json::json;

fn counters() -> Vec<CounterConfig> {
    Config::from_json(&json!({
        "metrics": {
            "cpu": {"description": "Processor time.", "unit": "%", "gauge": {}},
            "private": {
                "description": "Private bytes.",
                "unit": "By",
                "sum": {"aggregation": "cumulative"}
            }
        },
        "perfcounters": [
            {
                "object": "Processor",
                "instances": "*",
                "counters": [{
                    "name": "% Processor Time",
                    "metric": "cpu",
                    "attributes": {"host.role": "web"}
                }]
            },
            {
                "object": "Process",
                "instances": "app",
                "counters": [{"name": "Private Bytes", "metric": "private"}]
            },
            {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
        ]
    }))
    .unwrap()
    .counters
}

fn point(counter: usize, instance: Option<&str>, value: f64) -> Point {
    Point {
        counter,
        instance: instance.map(str::to_owned),
        value,
        time_unix_nano: 20,
        start_time_unix_nano: 10,
    }
}

fn attributes(attributes: &[KeyValue]) -> BTreeMap<&str, &str> {
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

fn double(point: &NumberDataPoint) -> f64 {
    match point.value {
        Some(number_data_point::Value::AsDouble(value)) => value,
        ref other => panic!("expected a double, got {other:?}"),
    }
}

fn find<'a>(metrics: &'a [Metric], name: &str) -> &'a Metric {
    metrics
        .iter()
        .find(|metric| metric.name == name)
        .unwrap_or_else(|| panic!("missing metric {name}"))
}

/// Scenario: A scrape produced no points.
/// Guarantees: Projection emits no batch rather than an empty one.
#[test]
fn omits_empty_samples() {
    assert!(project_points(&counters(), &[]).unwrap().is_none());
}

/// Scenario: One scrape holds wildcard Gauge points, an exact Sum point and a non-instanced Gauge.
/// Guarantees: Points carry the collection time and double values; only the Sum carries the
/// query-open start time as a cumulative non-monotonic Sum; static attributes and the returned
/// instance name are attached; non-instanced counters get no instance attribute; the resource
/// is tagged `os.type=windows`.
#[test]
fn projects_gauge_sum_and_attributes() {
    let counters = counters();
    let points = [
        point(0, Some("0"), 12.5),
        point(0, Some("_Total"), 6.25),
        point(1, Some("app"), 4096.0),
        point(2, None, 1.0e9),
    ];
    let request = decode(project_points(&counters, &points).unwrap().unwrap());
    let resource = &request.resource_metrics[0];
    assert_eq!(
        attributes(&resource.resource.as_ref().unwrap().attributes),
        BTreeMap::from([("os.type", "windows")])
    );
    let metrics = &resource.scope_metrics[0].metrics;
    assert_eq!(metrics.len(), 3);

    let cpu = find(metrics, "cpu");
    assert_eq!(cpu.unit, "%");
    let Some(metric::Data::Gauge(gauge)) = &cpu.data else {
        panic!("cpu should be a Gauge");
    };
    let cpu_points: Vec<_> = gauge
        .data_points
        .iter()
        .map(|point| {
            assert_eq!(point.time_unix_nano, 20);
            assert_eq!(point.start_time_unix_nano, 0);
            (attributes(&point.attributes), double(point))
        })
        .collect();
    assert_eq!(
        cpu_points,
        [
            (
                BTreeMap::from([("host.role", "web"), (INSTANCE_ATTRIBUTE, "0")]),
                12.5
            ),
            (
                BTreeMap::from([("host.role", "web"), (INSTANCE_ATTRIBUTE, "_Total")]),
                6.25
            ),
        ]
    );

    let private = find(metrics, "private");
    let Some(metric::Data::Sum(sum)) = &private.data else {
        panic!("private should be a Sum");
    };
    assert_eq!(
        sum.aggregation_temporality,
        AGGREGATION_TEMPORALITY_CUMULATIVE
    );
    assert!(!sum.is_monotonic);
    let sum_point = &sum.data_points[0];
    assert_eq!(
        (sum_point.start_time_unix_nano, sum_point.time_unix_nano),
        (10, 20)
    );
    assert_eq!(
        attributes(&sum_point.attributes),
        BTreeMap::from([(INSTANCE_ATTRIBUTE, "app")])
    );
    assert_eq!(double(sum_point), 4096.0);

    let memory = find(metrics, r"\Memory\Available Bytes");
    let Some(metric::Data::Gauge(gauge)) = &memory.data else {
        panic!("memory should be a Gauge");
    };
    assert!(gauge.data_points[0].attributes.is_empty());
    assert_eq!(double(&gauge.data_points[0]), 1.0e9);
}

/// Scenario: Sums are configured with each temporality and monotonic flag.
/// Guarantees: The Sum carries the configured temporality and monotonic flag, and the value is unchanged;
/// only a cumulative Sum carries the query-open start time, as Go sets none for delta.
#[test]
fn projects_sum_temporality_and_monotonic_flags() {
    for (aggregation, monotonic, temporality) in [
        ("cumulative", true, AGGREGATION_TEMPORALITY_CUMULATIVE),
        ("delta", false, AGGREGATION_TEMPORALITY_DELTA),
        ("delta", true, AGGREGATION_TEMPORALITY_DELTA),
    ] {
        let counters = Config::from_json(&json!({
            "metrics": {"m": {"sum": {"aggregation": aggregation, "monotonic": monotonic}}},
            "perfcounters": [{"object": "System", "counters": [{"name": "Threads", "metric": "m"}]}]
        }))
        .unwrap()
        .counters;
        let request = decode(
            project_points(&counters, &[point(0, None, 7.5)])
                .unwrap()
                .unwrap(),
        );
        let Some(metric::Data::Sum(sum)) =
            &request.resource_metrics[0].scope_metrics[0].metrics[0].data
        else {
            panic!("expected a Sum");
        };
        assert_eq!(sum.aggregation_temporality, temporality);
        assert_eq!(sum.is_monotonic, monotonic);
        assert_eq!(double(&sum.data_points[0]), 7.5);
        let start = if temporality == AGGREGATION_TEMPORALITY_CUMULATIVE {
            10
        } else {
            0
        };
        assert_eq!(sum.data_points[0].start_time_unix_nano, start);
    }
}

/// Scenario: A counter configures a static `instance` attribute and its points carry returned instance names.
/// Guarantees: The static value replaces the generated instance attribute.
#[test]
fn static_instance_attribute_overrides_generated() {
    let counters = Config::from_json(&json!({
        "perfcounters": [{
            "object": "Processor",
            "instances": "0",
            "counters": [{"name": "% Processor Time", "attributes": {"instance": "fixed"}}]
        }]
    }))
    .unwrap()
    .counters;
    let request = decode(
        project_points(&counters, &[point(0, Some("0"), 1.0)])
            .unwrap()
            .unwrap(),
    );
    let Some(metric::Data::Gauge(gauge)) =
        &request.resource_metrics[0].scope_metrics[0].metrics[0].data
    else {
        panic!("expected a Gauge");
    };
    assert_eq!(
        attributes(&gauge.data_points[0].attributes),
        BTreeMap::from([(INSTANCE_ATTRIBUTE, "fixed")])
    );
    assert_eq!(gauge.data_points[0].attributes.len(), 1);
}
