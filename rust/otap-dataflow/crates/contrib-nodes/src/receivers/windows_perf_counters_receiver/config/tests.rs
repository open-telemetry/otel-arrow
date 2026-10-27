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

/// Scenario: Exact object counters use no instance or explicitly named instances.
/// Guarantees: Each path is built from object, instance and counter, and keeps its metric kind and metadata.
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
                "object": "Process",
                "instances": ["app", "worker"],
                "counters": [{
                    "name": "Private Bytes",
                    "metric": "private",
                    "attributes": {"role": "app"},
                    "scale_power10": -3
                }]
            }
        ]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 3);
    let memory = &config.counters[0];
    assert_eq!(memory.path, r"\Memory\Available Bytes");
    assert_eq!(memory.aggregate.as_deref(), Some("_Total"));
    assert_eq!(memory.metric_kind, MetricKind::Gauge);
    assert_eq!(memory.unit.as_ref(), "By");
    let app = &config.counters[1];
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
    assert_eq!(config.counters[2].path, r"\Process(worker)\Private Bytes");
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

/// Scenario: perfcounters is empty, or an object is empty, has no counters, or lists an empty
/// instance, alone, beside `*`, or from comma-splitting a scalar ("0,").
/// Guarantees: Each is rejected at load with the field, as Go's validation does.
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
            "perfcounters[0].instances must not contain an empty name",
        ),
        (
            json!({"object": "Process", "instances": ["*", ""], "counters": [{"name": "ID Process"}]}),
            "perfcounters[0].instances must not contain an empty name",
        ),
        (
            json!({"object": "Process", "instances": "0,", "counters": [{"name": "ID Process"}]}),
            "perfcounters[0].instances must not contain an empty name",
        ),
    ] {
        assert_config_error(json!({"perfcounters": [object]}), expected);
    }
}

/// Scenario: Collection timing uses sub-second and multi-day durations or a zero collection interval.
/// Guarantees: Positive intervals and delays are accepted; a zero collection interval is rejected.
#[test]
fn enforces_collection_timing_bounds() {
    for (interval, delay) in [("500ms", "0s"), ("86401s", "0s"), ("1s", "86401s")] {
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
}

/// Scenario: Configuration contains an unknown field or a field removed from the earlier engine.
/// Guarantees: Deserialization rejects each one instead of ignoring it.
#[test]
fn rejects_unknown_and_removed_fields() {
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

/// Scenario: timeout is omitted, zero, positive or negative.
/// Guarantees: Omitted and zero select the next-tick default, positive is kept, negative is rejected.
#[test]
fn normalizes_timeout() {
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

/// Scenario: A metric has no kind key, a null `sum`, a bare `gauge`, or an empty `gauge` map; then
/// a `gauge` is combined with an empty `sum` and with a complete `sum`.
/// Guarantees: Without a present `sum` mapping the metric is a Gauge; `gauge` with any present `sum`
/// mapping is rejected.
#[test]
fn gauge_is_default_and_conflicts_with_any_sum() {
    for metric in [
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

/// Scenario: Two entries name the same path in different case with identical metric and attributes,
/// and other entries reuse a path with a different metric or different attributes.
/// Guarantees: Only fully identical entries are rejected; differing metric or attributes are kept.
#[test]
fn rejects_only_identical_duplicate_entries() {
    assert_config_error(
        json!({
            "perfcounters": [
                {"object": "Memory", "counters": [{"name": "Available Bytes", "metric": "a"}]},
                {"object": "memory", "counters": [{"name": "available bytes", "metric": "a"}]}
            ],
            "metrics": {"a": {"gauge": {}}}
        }),
        "perfcounters[1].counters[0]: duplicate counter path \\memory\\available bytes with the same metric and attributes",
    );
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

/// Scenario: A wildcard appears in an object or counter name, or the counter name is `*`.
/// Guarantees: Wildcards are accepted only in instance names, with a field-specific error.
#[test]
fn rejects_wildcards_outside_instances() {
    for (object, counter, expected) in [
        (
            "Proc*",
            "ID Process",
            "perfcounters[0].object: wildcards are supported only in instance names",
        ),
        (
            "Process",
            "ID ?rocess",
            "perfcounters[0].counters[0].name: wildcards are supported only in instance names",
        ),
        (
            "Process",
            "*",
            "perfcounters[0].counters[0].name: whole-counter wildcard \"*\" is not supported yet; list counter names explicitly",
        ),
    ] {
        assert_config_error(
            json!({"perfcounters": [{
                "object": object, "instances": "*", "counters": [{"name": counter}]
            }]}),
            expected,
        );
    }
}

/// Scenario: A backslash appears in the object, an instance (alone or beside `*`), or the counter
/// name.
/// Guarantees: Remote or malformed paths are rejected at load with a field-specific error.
#[test]
fn rejects_backslash_in_path_components() {
    for (object, instances, counter, field) in [
        (
            r"\\host\Processor",
            json!("*"),
            "% Processor Time",
            "perfcounters[0].object",
        ),
        (
            "Processor",
            json!(r"0)\x"),
            "% Processor Time",
            "perfcounters[0].instances",
        ),
        (
            "Processor",
            json!(["*", r"a\b"]),
            "% Processor Time",
            "perfcounters[0].instances",
        ),
        (
            "Processor",
            json!("*"),
            r"x\% Processor Time",
            "perfcounters[0].counters[0].name",
        ),
    ] {
        assert_config_error(
            json!({"perfcounters": [{
                "object": object, "instances": instances, "counters": [{"name": counter}]
            }]}),
            &format!("{field}: remote or malformed counter paths are not supported"),
        );
    }
}

/// Scenario: A metric definition is explicit null, as `name:` with no value in YAML.
/// Guarantees: It loads as a default Gauge with unit "1" and no description, the same as `{}`.
#[test]
fn null_metric_definition_is_gauge() {
    let mut value = config_with_timing("30s", "1s");
    value["metrics"]["available"] = serde_json::Value::Null;
    let counter = &Config::from_json(&value).unwrap().counters[0];
    assert_eq!(counter.metric_kind, MetricKind::Gauge);
    assert_eq!((&*counter.unit, &*counter.description), ("1", ""));
}

/// Scenario: Each optional scalar option (description, unit, sum.monotonic, attributes,
/// recreate_query, aggregation_name, initial_delay) is explicit null, or `instances` is "".
/// Guarantees: Null takes the zero value and then any existing default, as the Collector's
/// confmap decodes it; an empty scalar `instances` selects the non-instanced path.
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

/// Scenario: recreate_query is omitted, false, or true on one of two counters.
/// Guarantees: The flag is kept per counter and defaults to false.
#[test]
fn accepts_recreate_query() {
    let config = Config::from_json(&json!({"perfcounters": [{
        "object": "System",
        "counters": [
            {"name": "Processes"},
            {"name": "Threads", "recreate_query": false},
            {"name": "File Read Bytes/sec", "recreate_query": true}
        ]
    }]}))
    .unwrap();
    let flags: Vec<_> = config
        .counters
        .iter()
        .map(|counter| counter.recreate_query)
        .collect();
    assert_eq!(flags, [false, false, true]);
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
