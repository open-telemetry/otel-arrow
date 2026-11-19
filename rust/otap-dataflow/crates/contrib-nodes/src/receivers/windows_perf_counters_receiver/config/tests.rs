// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use serde_json::json;

fn metrics() -> serde_json::Value {
    json!({
        "available": {
            "description": "Available physical memory.",
            "unit": "By",
            "gauge": {}
        },
        "private": {
            "description": "Committed private memory.",
            "unit": "By",
            "sum": {"aggregation": "cumulative", "monotonic": false}
        }
    })
}

fn config_with_timing(collection_interval: &str, initial_delay: &str) -> serde_json::Value {
    json!({
        "metrics": {
            "available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }
        },
        "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available Bytes", "metric": "available"}]
        }],
        "collection_interval": collection_interval,
        "initial_delay": initial_delay
    })
}

fn assert_config_error(value: serde_json::Value, expected: &str) {
    let error = Config::from_json(&value).unwrap_err().to_string();
    assert!(error.contains(expected), "unexpected error: {error}");
}

fn gauge_config(perfcounter: serde_json::Value) -> serde_json::Value {
    json!({
        "metrics": {"available": {
            "description": "Available physical memory.",
            "unit": "By",
            "gauge": {}
        }},
        "perfcounters": [perfcounter]
    })
}

fn sum_config(sum: serde_json::Value) -> serde_json::Value {
    let mut value = config_with_timing("30s", "1s");
    value["metrics"]["available"] = json!({"unit": "By", "sum": sum});
    value
}

/// Scenario: Exact object counters use no instance or explicitly named instances, one of which
/// contains backslashes and balanced parentheses; one counter sets recreate_query and the others
/// omit it.
/// Guarantees: Each path is built from object, instance and counter, keeping such an instance
/// name verbatim, and keeps its metric kind, metadata and recreate_query flag, which
/// defaults to false.
#[test]
fn normalizes_exact_configuration() {
    let config = Config::from_json(&json!({
        "metrics": metrics(),
        "perfcounters": [
            {
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "available"}]
            },
            {
                "object": "Paging File",
                "instances": r"D:\swap (1)\pagefile.sys",
                "counters": [{"name": "% Usage", "metric": "available"}]
            },
            {
                "object": "Process",
                "instances": ["app", "worker"],
                "counters": [{
                    "name": "Private Bytes",
                    "metric": "private",
                    "attributes": {"role": "app"},
                    "scale_power10": -3,
                    "recreate_query": true
                }]
            }
        ]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 4);
    let memory = &config.counters[0];
    assert_eq!(memory.path, r"\Memory\Available Bytes");
    assert_eq!(memory.aggregate.as_deref(), Some("_Total"));
    assert_eq!(memory.metric_kind, MetricKind::Gauge);
    assert_eq!(memory.unit.as_ref(), "By");
    assert!(!memory.recreate_query);
    assert_eq!(
        config.counters[1].path,
        r"\Paging File(D:\swap (1)\pagefile.sys)\% Usage"
    );
    let app = &config.counters[2];
    assert_eq!(app.path, r"\Process(app)\Private Bytes");
    assert_eq!(
        app.metric_kind,
        MetricKind::Sum {
            temporality: Temporality::Cumulative,
            monotonic: false
        }
    );
    assert_eq!(app.name.as_ref(), "private");
    assert_eq!(app.attributes.get("role").map(String::as_str), Some("app"));
    assert_eq!(app.scale_power10, -3);
    assert!(app.recreate_query);
    assert_eq!(config.counters[3].path, r"\Process(worker)\Private Bytes");
    assert_eq!(config.collection_interval, Duration::from_secs(60));
    assert_eq!(config.initial_delay, Duration::from_secs(1));
}

/// Scenario: Instances are omitted, empty, partial `*` or `?` patterns, `*` mixed with names, `*`
/// with the aggregate listed, or the aggregate alone, under default and custom aggregation names;
/// a scalar is a comma-separated list, or a list entry contains a comma.
/// Guarantees: Omitted and empty select the non-instanced path; every other name is its own path;
/// a scalar is split on commas without trimming, while list entries stay literal;
/// `*` collapses other names and opts the aggregate in only when the aggregate is also
/// listed; otherwise reads clear or drop the configured aggregate name.
#[test]
fn normalizes_instance_selection_forms() {
    let paths = |instances: serde_json::Value, aggregation_name: &str| {
        let mut object = json!({
            "object": "Process",
            "aggregation_name": aggregation_name,
            "counters": [{"name": "ID Process"}]
        });
        if !instances.is_null() {
            object["instances"] = instances;
        }
        Config::from_json(&json!({"perfcounters": [object]}))
            .unwrap()
            .counters
            .into_iter()
            .map(|counter| (counter.path, counter.aggregate.map(|name| name.to_string())))
            .collect::<Vec<_>>()
    };
    let path = |instance: &str| format!(r"\Process({instance})\ID Process");
    let total = || Some("_Total".to_owned());
    let none = || (r"\Process\ID Process".to_owned(), total());
    assert_eq!(paths(json!(null), ""), [none()]);
    assert_eq!(paths(json!([]), ""), [none()]);
    assert_eq!(paths(json!("svc*"), ""), [(path("svc*"), total())]);
    assert_eq!(
        paths(json!(["svchos?", "app"]), ""),
        [(path("svchos?"), total()), (path("app"), total())]
    );
    assert_eq!(paths(json!(["app", "*"]), ""), [(path("*"), total())]);
    assert_eq!(paths(json!(["*", "_Total"]), ""), [(path("*"), None)]);
    assert_eq!(paths(json!("_Total"), ""), [(path("_Total"), total())]);
    assert_eq!(
        paths(json!(["*", "_Total"]), "Sum"),
        [(path("*"), Some("Sum".to_owned()))]
    );
    assert_eq!(paths(json!(["*", "Sum"]), "Sum"), [(path("*"), None)]);
    assert_eq!(
        paths(json!("0,_Total"), ""),
        [(path("0"), total()), (path("_Total"), total())]
    );
    assert_eq!(
        paths(json!(["0,_Total"]), ""),
        [(path("0,_Total"), total())]
    );
    assert_eq!(paths(json!("*,_Total"), ""), [(path("*"), None)]);
}

/// Scenario: perfcounters is empty, or an object is empty, has no counters, lists an empty
/// instance (alone, beside `*`, or from comma-splitting a scalar "0,"), has an empty counter name,
/// or has a static attribute with an empty key.
/// Guarantees: Each is rejected at load naming the field, so no unusable path is collected and no
/// point carries an empty attribute key.
#[test]
fn rejects_empty_required_fields() {
    assert_config_error(
        json!({"perfcounters": []}),
        "perfcounters must not be empty",
    );
    for (object, expected) in [
        (
            json!({"object": "", "counters": [{"name": "Threads"}]}),
            "perfcounters[0].object must not be empty",
        ),
        (
            json!({"object": "System", "counters": []}),
            "perfcounters[0].counters must not be empty",
        ),
        (
            json!({"object": "Process", "instances": ["app", ""], "counters": [{"name": "ID Process"}]}),
            "perfcounters[0].instances[1] must not be empty",
        ),
        (
            json!({"object": "Process", "instances": ["*", ""], "counters": [{"name": "ID Process"}]}),
            "perfcounters[0].instances[1] must not be empty",
        ),
        (
            json!({"object": "Process", "instances": "0,", "counters": [{"name": "ID Process"}]}),
            "perfcounters[0].instances[1] must not be empty",
        ),
        (
            json!({"object": "Memory", "counters": [{"name": ""}]}),
            "perfcounters[0].counters[0].name must not be empty",
        ),
        (
            json!({"object": "Memory", "counters": [
                {"name": "Available Bytes", "attributes": {"": "value"}}
            ]}),
            "perfcounters[0].counters[0].attributes: attribute keys must not be empty",
        ),
    ] {
        assert_config_error(json!({"perfcounters": [object]}), expected);
    }
}

/// Scenario: Collection timing uses sub-second and 24h durations, a zero collection interval, or a
/// collection_interval, initial_delay or timeout just over 24h; timeout is omitted, zero, positive
/// or negative.
/// Guarantees: Positive durations up to 24h are accepted; a zero collection interval is rejected;
/// a longer duration is rejected naming its field, instead of overflowing the schedule at run time;
/// an omitted or zero timeout selects the next-tick default and a negative one is rejected.
#[test]
fn enforces_collection_timing_bounds() {
    for (interval, delay) in [("500ms", "0s"), ("24h", "0s"), ("1s", "24h")] {
        let config = Config::from_json(&config_with_timing(interval, delay)).unwrap();
        assert_eq!(
            config.collection_interval,
            humantime::parse_duration(interval).unwrap()
        );
        assert_eq!(
            config.initial_delay,
            humantime::parse_duration(delay).unwrap()
        );
    }
    assert_config_error(
        config_with_timing("0s", "0s"),
        "collection_interval must be greater than zero",
    );
    for field in ["collection_interval", "initial_delay", "timeout"] {
        let mut config = config_with_timing("30s", "1s");
        config[field] = json!("24h 1s");
        assert_config_error(config, &format!("{field} must not exceed 24h"));
    }
    for (timeout, expected) in [
        (None, None),
        (Some("0s"), None),
        (Some("2s"), Some(Duration::from_secs(2))),
    ] {
        let mut value = config_with_timing("30s", "1s");
        if let Some(timeout) = timeout {
            value["timeout"] = json!(timeout);
        }
        assert_eq!(Config::from_json(&value).unwrap().timeout, expected);
    }
    let mut value = config_with_timing("30s", "1s");
    value["timeout"] = json!("-1s");
    assert!(Config::from_json(&value).is_err());
}

/// Scenario: Configuration contains an unknown field, including the unsupported expansion-limit
/// options.
/// Guarantees: Deserialization rejects each one instead of ignoring it.
#[test]
fn rejects_unknown_fields() {
    for field in [
        "unknown",
        "wildcard_refresh_interval",
        "max_instances_per_wildcard",
        "max_expanded_counters",
    ] {
        let mut value = config_with_timing("30s", "1s");
        value[field] = json!("1s");
        assert_config_error(value, &format!("unknown field `{field}`"));
    }
}

/// Scenario: A Sum selects cumulative or delta aggregation, monotonic or not, or omits monotonic.
/// Guarantees: Every combination is accepted with its temporality and flag; monotonic defaults to false.
#[test]
fn accepts_sum_temporality_and_monotonicity() {
    for (sum, temporality, monotonic) in [
        (
            json!({"aggregation": "cumulative"}),
            Temporality::Cumulative,
            false,
        ),
        (
            json!({"aggregation": "cumulative", "monotonic": true}),
            Temporality::Cumulative,
            true,
        ),
        (json!({"aggregation": "delta"}), Temporality::Delta, false),
        (
            json!({"aggregation": "delta", "monotonic": true}),
            Temporality::Delta,
            true,
        ),
    ] {
        let config = Config::from_json(&sum_config(sum)).unwrap();
        assert_eq!(
            config.counters[0].metric_kind,
            MetricKind::Sum {
                temporality,
                monotonic
            }
        );
    }
}

/// Scenario: A metric definition is explicit null (`name:` with no value in YAML), or has no kind
/// key, a null `sum`, a bare `gauge`, or an empty `gauge` map; then
/// a `gauge` is combined with an empty `sum` and with a complete `sum`.
/// Guarantees: Without a present `sum` mapping the metric is a Gauge; `gauge` with any present `sum`
/// mapping is rejected.
#[test]
fn gauge_is_default_and_conflicts_with_any_sum() {
    for metric in [
        json!(null),
        json!({}),
        json!({"sum": null}),
        json!({"gauge": null}),
        json!({"gauge": {}}),
    ] {
        let mut value = sum_config(json!(null));
        value["metrics"]["available"] = metric;
        let config = Config::from_json(&value).unwrap();
        assert_eq!(config.counters[0].metric_kind, MetricKind::Gauge);
    }
    for sum in [json!({}), json!({"aggregation": "cumulative"})] {
        let mut value = sum_config(sum);
        value["metrics"]["available"]["gauge"] = json!({});
        assert_eq!(
            Config::from_json(&value).unwrap_err().to_string(),
            "An invalid user configuration occurred: metrics.available must not contain both gauge and sum"
        );
    }
}

/// Scenario: A present `sum` mapping is empty, sets only `monotonic`, or names an invalid aggregation.
/// Guarantees: A `sum` without `aggregation` is rejected instead of becoming a Gauge, and an
/// unknown aggregation is rejected.
#[test]
fn sum_requires_aggregation() {
    for (sum, expected) in [
        (json!({}), "metrics.available.sum requires aggregation"),
        (
            json!({"monotonic": true}),
            "metrics.available.sum requires aggregation",
        ),
        (
            json!({"monotonic": false}),
            "metrics.available.sum requires aggregation",
        ),
        (json!({"aggregation": "invalid"}), "unknown variant"),
    ] {
        assert_config_error(sum_config(sum), expected);
    }
}

/// Scenario: Metric definitions are omitted or empty and metric references are omitted or empty.
/// Guarantees: The counter becomes a Gauge named by its path with unit "1", keeping its own attributes and scale.
#[test]
fn defaults_path_metrics_without_definitions() {
    for definitions in [None, Some(json!({}))] {
        for reference in [None, Some("")] {
            let mut value = json!({"perfcounters": [{
                "object": "Process", "instances": ["app"],
                "counters": [{
                    "name": "Private Bytes",
                    "attributes": {"role": "app"},
                    "scale_power10": -3
                }]
            }]});
            if let Some(definitions) = &definitions {
                value["metrics"] = definitions.clone();
            }
            if let Some(reference) = reference {
                value["perfcounters"][0]["counters"][0]["metric"] = json!(reference);
            }
            let config = Config::from_json(&value).unwrap();
            let counter = &config.counters[0];
            assert_eq!(counter.name.as_ref(), r"\Process(app)\Private Bytes");
            assert_eq!(counter.metric_kind, MetricKind::Gauge);
            assert_eq!(counter.unit.as_ref(), "1");
            assert!(counter.description.is_empty());
            assert_eq!(
                counter.attributes.get("role").map(String::as_str),
                Some("app")
            );
            assert_eq!(counter.scale_power10, -3);
        }
    }
}

/// Scenario: A counter mapping names a metric that is not defined.
/// Guarantees: Validation identifies the public YAML field that contains the bad reference.
#[test]
fn rejects_undefined_metric_reference() {
    assert_config_error(
        gauge_config(json!({
            "object": "Memory",
            "counters": [{"name": "Available Bytes", "metric": "missing"}]
        })),
        "perfcounters[0].counters[0].metric references undefined metric \"missing\"",
    );
}

/// Builds a config whose objects all map their counters to the Gauge `m`.
fn shared_metric(objects: serde_json::Value) -> serde_json::Value {
    json!({"metrics": {"m": {"gauge": {}}}, "perfcounters": objects})
}

fn object(
    name: &str,
    instances: serde_json::Value,
    counters: serde_json::Value,
) -> serde_json::Value {
    let mut object = json!({"object": name, "counters": counters});
    if !instances.is_null() {
        object["instances"] = instances;
    }
    object
}

fn counter(name: &str, attributes: serde_json::Value) -> serde_json::Value {
    json!([{"name": name, "metric": "m", "attributes": attributes}])
}

/// Scenario: Entries that share a metric and static attributes are the same path in different
/// case; different counters; a wildcard beside another counter's named instance; instances "a"
/// and "A"; instances "PING" beside "PING#0" or "PING#00", or "PING#1" beside "PING#01", which
/// PDH reads as the same instance; or a lone
/// aggregate instance beside an object without instances. Separately, a static
/// `instance` is set on a wildcard instance.
/// Guarantees: Each configuration that could emit indistinguishable points is rejected at load,
/// naming both fields and paths, the metric and the fix; a static `instance` on a wildcard is
/// rejected with its own fix.
#[test]
fn rejects_indistinguishable_points() {
    let none = json!(null);
    let memory = |name: &str| object("Memory", none.clone(), counter(name, json!({})));
    for (objects, expected) in [
        (
            json!([
                memory("Available Bytes"),
                object(
                    "memory",
                    none.clone(),
                    counter("available bytes", json!({}))
                )
            ]),
            r#"perfcounters[0].counters[0] (\Memory\Available Bytes) and perfcounters[1].counters[0] (\memory\available bytes) would emit indistinguishable points for metric "m"; give one of them a static attribute with a distinct value"#,
        ),
        (
            json!([memory("Available Bytes"), memory("Committed Bytes")]),
            r"perfcounters[0].counters[0] (\Memory\Available Bytes) and perfcounters[1].counters[0] (\Memory\Committed Bytes)",
        ),
        (
            json!([
                object(
                    "Processor",
                    json!("*"),
                    counter("% Processor Time", json!({}))
                ),
                object("Processor", json!("0"), counter("% Idle Time", json!({})))
            ]),
            r"perfcounters[0].counters[0] (\Processor(*)\% Processor Time) and perfcounters[1].counters[0] (\Processor(0)\% Idle Time)",
        ),
        (
            json!([
                object("Process", json!("a"), counter("ID Process", json!({}))),
                object("Process", json!("A"), counter("ID Process", json!({})))
            ]),
            r"perfcounters[0].counters[0] (\Process(a)\ID Process) and perfcounters[1].counters[0] (\Process(A)\ID Process)",
        ),
        (
            json!([object(
                "Process",
                json!(["PING", "PING#0"]),
                counter("ID Process", json!({}))
            )]),
            r"perfcounters[0].counters[0] (\Process(PING)\ID Process) and perfcounters[0].counters[0] (\Process(PING#0)\ID Process)",
        ),
        (
            json!([object(
                "Process",
                json!(["PING", "PING#00"]),
                counter("ID Process", json!({}))
            )]),
            r"perfcounters[0].counters[0] (\Process(PING)\ID Process) and perfcounters[0].counters[0] (\Process(PING#00)\ID Process)",
        ),
        (
            json!([object(
                "Process",
                json!(["PING#1", "PING#01"]),
                counter("ID Process", json!({}))
            )]),
            r"perfcounters[0].counters[0] (\Process(PING#1)\ID Process) and perfcounters[0].counters[0] (\Process(PING#01)\ID Process)",
        ),
        (
            json!([
                object(
                    "Processor",
                    json!(["_Total"]),
                    counter("% Processor Time", json!({}))
                ),
                memory("Available Bytes")
            ]),
            r"perfcounters[0].counters[0] (\Processor(_Total)\% Processor Time) and perfcounters[1].counters[0] (\Memory\Available Bytes)",
        ),
        (
            json!([object(
                "Processor",
                json!("*"),
                counter("% Processor Time", json!({"instance": "cpu"}))
            )]),
            "perfcounters[0].counters[0].attributes.instance: a static instance on wildcard instances collapses every instance into one series; remove the static instance or select a specific instance",
        ),
    ] {
        assert_config_error(shared_metric(objects), expected);
    }
}

/// Scenario: A counter without `metric` emits under its path name, and a `metrics` entry with that
/// name is a cumulative Sum, a Gauge with another unit, or a Gauge with a description, used by a
/// counter with a distinct static attribute.
/// Guarantees: One metric name has one definition: each is rejected, naming both counters, the
/// differing field and the fix.
#[test]
fn rejects_conflicting_metric_definitions() {
    let path = r"\Memory\Available Bytes";
    for (definition, differs) in [
        (
            json!({"unit": "1", "sum": {"aggregation": "cumulative"}}),
            "gauge or sum",
        ),
        (json!({"unit": "By"}), "unit"),
        (json!({"description": "Available memory."}), "description"),
    ] {
        assert_config_error(
            json!({
                "metrics": {path: definition},
                "perfcounters": [
                    {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
                    {"object": "Memory", "counters": [
                        {"name": "Committed Bytes", "metric": path, "attributes": {"a": "b"}}
                    ]}
                ]
            }),
            &format!(
                r#"perfcounters[0].counters[0] and perfcounters[1].counters[0] use metric "\\Memory\\Available Bytes" with a different {differs}; give one of them a different metric name"#
            ),
        );
    }
}

/// Scenario: Entries share a metric but differ in a static attribute, in instance ("a" and "b"),
/// or in a static `instance` on a named instance and an object without instances; a lone
/// aggregate instance under a custom aggregation name sits beside an object without instances;
/// one path maps to different metrics or attributes.
/// Guarantees: Distinguishable points are accepted, so several counters can share a metric.
#[test]
fn accepts_distinguishable_points() {
    let none = json!(null);
    let mut custom = object(
        "Processor",
        json!(["_Total"]),
        counter("% Processor Time", json!({})),
    );
    custom["aggregation_name"] = json!("Sum");
    for objects in [
        json!([
            object(
                "Memory",
                none.clone(),
                counter("Available Bytes", json!({"state": "free"}))
            ),
            object(
                "Memory",
                none.clone(),
                counter("Committed Bytes", json!({"state": "used"}))
            )
        ]),
        json!([object(
            "Process",
            json!(["a", "b"]),
            counter("ID Process", json!({}))
        )]),
        json!([
            object(
                "Processor",
                json!("0"),
                counter("% Processor Time", json!({"instance": "x"}))
            ),
            object(
                "Memory",
                none.clone(),
                counter("Available Bytes", json!({"instance": "y"}))
            )
        ]),
        json!([
            custom,
            object(
                "Memory",
                none.clone(),
                counter("Available Bytes", json!({}))
            )
        ]),
    ] {
        assert!(
            Config::from_json(&shared_metric(objects.clone())).is_ok(),
            "{objects}"
        );
    }
    let config = Config::from_json(&json!({
        "metrics": {"a": {"gauge": {}}, "b": {"gauge": {}}},
        "perfcounters": [{
            "object": "Memory",
            "counters": [
                {"name": "Available Bytes", "metric": "a"},
                {"name": "Available Bytes", "metric": "b"},
                {"name": "Available Bytes", "metric": "b", "attributes": {"k": "v"}}
            ]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 3);
}

/// Scenario: An object or counter name contains a wildcard, the counter name is `*`, an object
/// names an instance in parentheses, a backslash appears in the object or counter name, an
/// instance closes a parenthesis it did not open or leaves one open, or a NUL appears in the
/// object, an instance, or the counter name.
/// Guarantees: Each path component accepts only its own role, with a field-specific error: wildcards
/// only in instances, instances only through `instances`, no instance that spills into the counter
/// name, and no remote or malformed paths.
#[test]
fn rejects_misplaced_path_components() {
    const MALFORMED: &str = "remote or malformed counter paths are not supported";
    for (object, instances, counter, field, message) in [
        (
            "Proc*",
            json!("*"),
            "ID Process",
            "object",
            "wildcards are supported only in instance names",
        ),
        (
            "Process",
            json!("*"),
            "ID ?rocess",
            "counters[0].name",
            "wildcards are supported only in instance names",
        ),
        (
            "Process",
            json!("*"),
            "*",
            "counters[0].name",
            "whole-counter wildcard \"*\" is not supported yet; list counter names explicitly",
        ),
        (
            "Processor(0)",
            json!(null),
            "% Processor Time",
            "object",
            "select instances with `instances`, not parentheses in the object name",
        ),
        (
            r"\\host\Processor",
            json!("*"),
            "% Processor Time",
            "object",
            MALFORMED,
        ),
        (
            "Processor",
            json!(r"0)\x"),
            "% Processor Time",
            "instances[0]",
            "parentheses in an instance name must be balanced",
        ),
        (
            "Processor",
            json!(["*", "a(b"]),
            "% Processor Time",
            "instances[1]",
            "parentheses in an instance name must be balanced",
        ),
        (
            "Mem\0ory",
            json!(null),
            "Available Bytes",
            "object",
            MALFORMED,
        ),
        (
            "Process",
            json!(["a\0b"]),
            "ID Process",
            "instances[0]",
            MALFORMED,
        ),
        (
            "Memory",
            json!(null),
            "Avail\0able Bytes",
            "counters[0].name",
            MALFORMED,
        ),
        (
            "Processor",
            json!("*"),
            r"x\% Processor Time",
            "counters[0].name",
            MALFORMED,
        ),
    ] {
        assert_config_error(
            json!({"perfcounters": [{
                "object": object, "instances": instances, "counters": [{"name": counter}]
            }]}),
            &format!("perfcounters[0].{field}: {message}"),
        );
    }
}

/// Scenario: Each optional scalar option (description, unit, sum.monotonic, attributes,
/// recreate_query, aggregation_name, initial_delay) is explicit null, or `instances` is "".
/// Guarantees: Null takes the zero value and then any existing default; an empty scalar `instances` selects the non-instanced path.
#[test]
fn null_options_and_empty_scalar_instances_use_defaults() {
    let base = json!({
        "metrics": {"m": {
            "description": "d",
            "unit": "By",
            "sum": {"aggregation": "cumulative", "monotonic": true}
        }},
        "perfcounters": [{
            "object": "Process",
            "instances": "app",
            "aggregation_name": "All",
            "counters": [{
                "name": "ID Process",
                "metric": "m",
                "recreate_query": true,
                "attributes": {"k": "v"}
            }]
        }],
        "initial_delay": "5s"
    });
    fn counter(config: &Config) -> &CounterConfig {
        &config.counters[0]
    }
    let cases: [(&str, serde_json::Value, fn(&Config)); 8] = [
        ("/metrics/m/description", json!(null), |c| {
            assert_eq!(&*counter(c).description, "")
        }),
        ("/metrics/m/unit", json!(null), |c| {
            assert_eq!(&*counter(c).unit, "1")
        }),
        ("/metrics/m/sum/monotonic", json!(null), |c| {
            assert_eq!(
                counter(c).metric_kind,
                MetricKind::Sum {
                    temporality: Temporality::Cumulative,
                    monotonic: false
                }
            )
        }),
        ("/perfcounters/0/counters/0/attributes", json!(null), |c| {
            assert!(counter(c).attributes.is_empty())
        }),
        (
            "/perfcounters/0/counters/0/recreate_query",
            json!(null),
            |c| assert!(!counter(c).recreate_query),
        ),
        ("/perfcounters/0/aggregation_name", json!(null), |c| {
            assert_eq!(counter(c).aggregate.as_deref(), Some("_Total"))
        }),
        ("/initial_delay", json!(null), |c| {
            assert_eq!(c.initial_delay, Duration::ZERO)
        }),
        ("/perfcounters/0/instances", json!(""), |c| {
            assert_eq!(counter(c).path, r"\Process\ID Process")
        }),
    ];
    for (pointer, replacement, check) in cases {
        let mut value = base.clone();
        *value.pointer_mut(pointer).unwrap() = replacement;
        check(&Config::from_json(&value).unwrap_or_else(|e| panic!("{pointer}: {e}")));
    }
}

/// Scenario: Decimal scaling is configured at and beyond its bounds.
/// Guarantees: Inclusive bounds are retained; values outside them are rejected with the field.
#[test]
fn enforces_scaling_bounds() {
    let scaled = |scale_power10: i32| {
        gauge_config(json!({
            "object": "Memory",
            "counters": [{"name": "Available Bytes", "scale_power10": scale_power10}]
        }))
    };
    for scale_power10 in [MIN_SCALE_POWER10, MAX_SCALE_POWER10] {
        let config = Config::from_json(&scaled(scale_power10)).unwrap();
        assert_eq!(config.counters[0].scale_power10, scale_power10);
    }
    for scale_power10 in [MIN_SCALE_POWER10 - 1, MAX_SCALE_POWER10 + 1] {
        assert_config_error(
            scaled(scale_power10),
            "perfcounters[0].counters[0].scale_power10 must be between -18 and 18",
        );
    }
}
