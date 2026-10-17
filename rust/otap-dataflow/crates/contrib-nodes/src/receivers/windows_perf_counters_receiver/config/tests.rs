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
    let error = RuntimeConfig::from_json(&value).unwrap_err().to_string();
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

/// Scenario: Exact object counters use no instance or explicitly named instances.
/// Guarantees: Structured configuration normalizes paths and retains both metric kinds.
#[test]
fn normalizes_exact_configuration() {
    let config = RuntimeConfig::from_json(&json!({
        "metrics": metrics(),
        "perfcounters": [
            {
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "available"}]
            },
            {
                "object": "Process",
                "instances": ["app", "worker"],
                "counters": [{"name": "Private Bytes", "metric": "private"}]
            }
        ]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 3);
    assert_eq!(config.counters[0].path, r"\Memory\Available Bytes");
    assert_eq!(config.counters[0].kind, MetricKind::Gauge);
    assert_eq!(config.counters[1].path, r"\Process(app)\Private Bytes");
    assert_eq!(config.counters[1].kind, MetricKind::Sum);
    assert_eq!(config.collection_interval, Duration::from_secs(30));
    assert_eq!(config.initial_delay, Duration::from_secs(1));
}

/// Scenario: Collection timing is configured at, below, and above its supported boundaries.
/// Guarantees: Exact boundary values succeed while out-of-range values identify their field.
#[test]
fn enforces_collection_timing_bounds() {
    let _config = RuntimeConfig::from_json(&config_with_timing("24h", "24h")).unwrap();

    assert_config_error(config_with_timing("500ms", "0s"), "collection_interval");
    assert_config_error(
        config_with_timing("86401s", "0s"),
        "collection_interval must be between",
    );
    assert_config_error(
        config_with_timing("1s", "86401s"),
        "initial_delay must be between",
    );
}

/// Scenario: Configuration contains a field not defined by the receiver contract.
/// Guarantees: Deserialization rejects misspelled or unsupported fields instead of ignoring them.
#[test]
fn rejects_unknown_fields() {
    let mut value = config_with_timing("30s", "1s");
    value["unknown"] = json!(true);
    assert_config_error(value, "unknown field `unknown`");
}

/// Scenario: An object's counter list or explicit instance list is empty.
/// Guarantees: Validation rejects objects that would produce no counter paths.
#[test]
fn rejects_empty_object_collections() {
    assert_config_error(
        gauge_config(json!({"object": "Memory", "counters": []})),
        "perfcounters[0].counters must contain at least one entry",
    );
    assert_config_error(
        gauge_config(json!({
            "object": "Process",
            "instances": [],
            "counters": [{"name": "Private Bytes", "metric": "available"}]
        })),
        "perfcounters[0].instances must not be empty",
    );
}

/// Scenario: Required text values are empty or whitespace-only.
/// Guarantees: Validation identifies each blank field instead of emitting unnamed telemetry.
#[test]
fn rejects_blank_required_values() {
    assert_config_error(
        json!({
            "metrics": {" ": {"description": "Value.", "unit": "By", "gauge": {}}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": " "}]
            }]
        }),
        "metric name must not be empty",
    );
    assert_config_error(
        gauge_config(json!({
            "object": "Memory",
            "counters": [{
                "name": "Available Bytes",
                "metric": "available",
                "attributes": {" ": "value"}
            }]
        })),
        "attribute key must not be empty",
    );
}

/// Scenario: A metric definition selects no metric kind.
/// Guarantees: Omission selects Gauge without overriding metric metadata or collection timing.
#[test]
fn defaults_missing_metric_kind_to_gauge() {
    let config = RuntimeConfig::from_json(&json!({
        "metrics": {"value": {"description": "Value.", "unit": "1"}},
        "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available Bytes", "metric": "value"}]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters[0].kind, MetricKind::Gauge);
    assert_eq!(config.counters[0].unit, "1");
    assert_eq!(config.counters[0].description.as_ref(), "Value.");
    assert_eq!(config.collection_interval, Duration::from_secs(30));
}

/// Scenario: A cumulative Sum omits monotonicity or explicitly sets it to false.
/// Guarantees: Both forms normalize to the supported cumulative non-monotonic Sum.
#[test]
fn defaults_sum_monotonicity_to_false() {
    for sum in [
        json!({"aggregation": "cumulative"}),
        json!({"aggregation": "cumulative", "monotonic": false}),
    ] {
        let mut value = config_with_timing("45s", "2s");
        let _ = value["metrics"]["available"]
            .as_object_mut()
            .unwrap()
            .remove("gauge");
        value["metrics"]["available"]["sum"] = sum;
        let config = RuntimeConfig::from_json(&value).unwrap();
        assert_eq!(config.counters[0].kind, MetricKind::Sum);
        assert_eq!(config.collection_interval, Duration::from_secs(45));
        assert_eq!(config.initial_delay, Duration::from_secs(2));
    }
}

/// Scenario: Gauge is written as a bare YAML key or an empty map.
/// Guarantees: Both shorthands select Gauge and still conflict with an explicit Sum.
#[test]
fn accepts_null_metric_kind_shorthand() {
    for gauge in [json!(null), json!({})] {
        let mut value = config_with_timing("30s", "1s");
        value["metrics"]["available"]["gauge"] = gauge;
        let config = RuntimeConfig::from_json(&value).unwrap();
        assert_eq!(config.counters[0].kind, MetricKind::Gauge);
        value["metrics"]["available"]["sum"] = json!({"aggregation": "cumulative"});
        assert_config_error(value, "must not contain both gauge and sum");
    }
}

/// Scenario: A metric definition selects both supported metric kinds.
/// Guarantees: Validation rejects ambiguous projection semantics.
#[test]
fn rejects_ambiguous_metric_kind() {
    assert_config_error(
        json!({
            "metrics": {"value": {
                "description": "Value.",
                "unit": "1",
                "gauge": {},
                "sum": {"aggregation": "cumulative"}
            }},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "value"}]
            }]
        }),
        "metrics.value must not contain both gauge and sum",
    );
}

/// Scenario: Sum is null, empty, lacks aggregation or selects an invalid aggregation.
/// Guarantees: Sum never silently becomes Gauge or defaults its aggregation.
#[test]
fn rejects_missing_or_invalid_sum_aggregation() {
    for (sum, expected) in [
        (json!(null), "null Sum is invalid"),
        (json!({}), "missing field `aggregation`"),
        (json!({"monotonic": false}), "missing field `aggregation`"),
        (json!({"aggregation": null}), "expected string or map"),
        (json!({"aggregation": ""}), "unknown variant"),
        (json!({"aggregation": "invalid"}), "unknown variant"),
    ] {
        let mut value = config_with_timing("30s", "1s");
        let _ = value["metrics"]["available"]
            .as_object_mut()
            .unwrap()
            .remove("gauge");
        value["metrics"]["available"]["sum"] = sum;
        assert_config_error(value, expected);
    }
}

/// Scenario: Sum selects delta temporality or monotonic execution.
/// Guarantees: Unsupported selections fail with supported alternatives in the error.
#[test]
fn rejects_unsupported_sum_execution() {
    for (aggregation, monotonic, expected) in [
        ("delta", false, "delta Sum is not supported yet"),
        ("delta", true, "delta Sum is not supported yet"),
        (
            "cumulative",
            true,
            "monotonic Sum (monotonic: true) is not supported yet",
        ),
    ] {
        let mut value = config_with_timing("30s", "1s");
        let _ = value["metrics"]["available"]
            .as_object_mut()
            .unwrap()
            .remove("gauge");
        value["metrics"]["available"]["sum"] = json!({
            "aggregation": aggregation, "monotonic": monotonic
        });
        let error = RuntimeConfig::from_json(&value).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("metrics.available"), "{error}");
        assert!(
            error.contains("use gauge or sum with aggregation: cumulative and monotonic: false"),
            "{error}"
        );
    }
}

/// Scenario: A metric definition uses an unrecognized up_down_counter field.
/// Guarantees: Only gauge and sum are accepted as metric-kind fields.
#[test]
fn rejects_removed_metric_kind_alias() {
    let mut value = config_with_timing("30s", "1s");
    let _ = value["metrics"]["available"]
        .as_object_mut()
        .unwrap()
        .remove("gauge");
    value["metrics"]["available"]["up_down_counter"] = json!({});
    assert_config_error(value, "unknown field `up_down_counter`");
}

/// Scenario: Metric metadata fields are omitted and another definition is unused.
/// Guarantees: Referenced metadata receives defaults and unused definitions are allowed.
#[test]
fn defaults_optional_metric_metadata_and_allows_unused_definitions() {
    let config = RuntimeConfig::from_json(&json!({
        "metrics": {
            "used": {"gauge": {}},
            "unused": {}
        },
        "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available Bytes", "metric": "used"}]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 1);
    let counter = &config.counters[0];
    assert_eq!(counter.name, "used");
    assert_eq!(counter.unit, "1");
    assert!(counter.description.is_empty());
    assert_eq!(counter.kind, MetricKind::Gauge);
}

/// Scenario: A referenced metric omits or supplies a blank description.
/// Guarantees: Descriptions normalize to empty without changing the supplied unit.
#[test]
fn defaults_optional_metric_description() {
    for description in [None, Some(""), Some(" \t ")] {
        let mut value = config_with_timing("30s", "1s");
        match description {
            None => {
                let _ = value["metrics"]["available"]
                    .as_object_mut()
                    .unwrap()
                    .remove("description");
            }
            Some(description) => value["metrics"]["available"]["description"] = json!(description),
        }
        let config = RuntimeConfig::from_json(&value).unwrap();
        assert!(config.counters[0].description.is_empty());
        assert_eq!(config.counters[0].unit, "By");
    }
}

/// Scenario: A referenced metric omits its unit but provides a description.
/// Guarantees: The unit defaults to "1" without changing the supplied description.
#[test]
fn defaults_omitted_metric_unit() {
    let mut value = config_with_timing("30s", "1s");
    let _ = value["metrics"]["available"]
        .as_object_mut()
        .unwrap()
        .remove("unit");
    let config = RuntimeConfig::from_json(&value).unwrap();
    assert_eq!(config.counters[0].unit, "1");
    assert_eq!(
        config.counters[0].description.as_ref(),
        "Available physical memory."
    );
}

/// Scenario: A referenced metric supplies an empty or whitespace-only unit.
/// Guarantees: Blank units receive the same "1" default as an omitted unit.
#[test]
fn defaults_empty_metric_unit() {
    for unit in ["", " \t "] {
        let mut value = config_with_timing("30s", "1s");
        value["metrics"]["available"]["unit"] = json!(unit);
        let config = RuntimeConfig::from_json(&value).unwrap();
        assert_eq!(config.counters[0].unit, "1");
    }
}

/// Scenario: One counter mapping expands across several instances.
/// Guarantees: Expanded counters share metadata instead of cloning it per path.
#[test]
fn shares_metadata_across_expanded_counters() {
    let config = RuntimeConfig::from_json(&gauge_config(json!({
        "object": "Process",
        "instances": ["a", "b"],
        "counters": [{
            "name": "Private Bytes",
            "metric": "available",
            "attributes": {"state": "used"}
        }]
    })))
    .unwrap();
    let [first, second] = config.counters.as_slice() else {
        panic!("expected two expanded counters");
    };
    assert!(Arc::ptr_eq(&first.attributes, &second.attributes));
    assert!(Arc::ptr_eq(&first.description, &second.description));
}

/// Scenario: Metric names violate the OpenTelemetry instrument-name syntax.
/// Guarantees: Validation rejects them instead of emitting invalid metric names.
#[test]
fn rejects_invalid_metric_names() {
    let too_long = format!("a{}", "b".repeat(MAX_METRIC_NAME_LEN));
    for name in ["1available", "avail able", "\u{e9}", too_long.as_str()] {
        assert_config_error(
            json!({
                "metrics": {name: {"description": "Value.", "unit": "By", "gauge": {}}},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": name}]
                }]
            }),
            "must start with an ASCII letter",
        );
    }
    let at_limit = format!("a{}", "b".repeat(MAX_METRIC_NAME_LEN - 1));
    for name in ["system.memory_usage-1/s", at_limit.as_str()] {
        assert!(
            RuntimeConfig::from_json(&json!({
                "metrics": {name: {"description": "Value.", "unit": "By", "gauge": {}}},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": name}]
                }]
            }))
            .is_ok()
        );
    }
}

/// Scenario: Metric units violate the OpenTelemetry instrument-unit syntax.
/// Guarantees: Validation rejects non-ASCII, control-character, or overlong units.
#[test]
fn rejects_invalid_metric_units() {
    let unit_config = |unit: &str| {
        json!({
            "metrics": {"m": {"description": "Value.", "unit": unit, "gauge": {}}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "m"}]
            }]
        })
    };
    let too_long = "a".repeat(MAX_METRIC_UNIT_LEN + 1);
    for unit in ["\u{b5}s", "B\ty", too_long.as_str()] {
        assert_config_error(unit_config(unit), "metrics.m.unit must be printable ASCII");
    }
    let at_limit = "a".repeat(MAX_METRIC_UNIT_LEN);
    for unit in ["By", "{packets}/s", "1", at_limit.as_str()] {
        assert!(RuntimeConfig::from_json(&unit_config(unit)).is_ok());
    }
}

/// Scenario: Metric definitions exceed the configuration's entry limit.
/// Guarantees: The definition bound applies even when most definitions are unused.
#[test]
fn rejects_too_many_metric_definitions() {
    let metrics = (0..=MAX_COUNTERS)
        .map(|index| {
            (
                format!("m{index}"),
                json!({"description": "Value.", "unit": "By", "gauge": {}}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    assert_config_error(
        json!({
            "metrics": metrics,
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "m0"}]
            }]
        }),
        "metrics must contain at most 256 entries",
    );
}

/// Scenario: An unused definition selects an unsupported metric kind.
/// Guarantees: All definitions are validated even when no mapping references them.
#[test]
fn validates_unused_metric_definitions() {
    let mut value = config_with_timing("30s", "1s");
    value["metrics"]["unused"] = json!({"sum": {"aggregation": "delta"}});
    assert_config_error(value, "metrics.unused: delta Sum is not supported yet");
}

/// Scenario: Definitions are omitted or empty and metric references are omitted or blank.
/// Guarantees: Path-named Gauges retain configured index spelling and canonical native paths.
#[test]
fn defaults_path_metrics_without_definitions() {
    for definitions in [None, Some(json!({}))] {
        let mut value = json!({"perfcounters": [{
            "object": " Process ", "instances": [" app#00 ", "worker#001"],
            "counters": [{"name": " Private Bytes ", "scale_power10": -3}]
        }]});
        if let Some(definitions) = definitions {
            value["metrics"] = definitions;
        }
        for reference in [None, Some(""), Some(" \t ")] {
            if let Some(reference) = reference {
                value["perfcounters"][0]["counters"][0]["metric"] = json!(reference);
            }
            let config = RuntimeConfig::from_json(&value).unwrap();
            for (counter, name, path) in [
                (
                    &config.counters[0],
                    r"\Process(app#00)\Private Bytes",
                    r"\Process(app)\Private Bytes",
                ),
                (
                    &config.counters[1],
                    r"\Process(worker#001)\Private Bytes",
                    r"\Process(worker#1)\Private Bytes",
                ),
            ] {
                assert_eq!(counter.name, name);
                assert_eq!(counter.path, path);
                assert_eq!(counter.kind, MetricKind::Gauge);
                assert_eq!(counter.unit, "1");
                assert!(counter.description.is_empty());
                assert!(counter.attributes.is_empty());
                assert_eq!(counter.scale_power10, -3);
            }
        }
    }
}

/// Scenario: An implicitly matched path definition selects invalid metadata or an unsupported kind.
/// Guarantees: Path-name resolution never replaces an invalid definition with the default Gauge.
#[test]
fn validates_implicit_path_metric_definitions() {
    for (definition, expected) in [
        (
            json!({"sum": {"aggregation": "delta"}}),
            "delta Sum is not supported yet",
        ),
        (
            json!({"sum": {"aggregation": "cumulative", "monotonic": true}}),
            "monotonic Sum",
        ),
        (
            json!({"gauge": {}, "sum": {"aggregation": "cumulative"}}),
            "must not contain both",
        ),
        (json!({"sum": null}), "null Sum is invalid"),
        (json!({"unit": "\u{b5}s"}), "unit must be printable ASCII"),
    ] {
        assert_config_error(
            json!({"metrics": {r"\Memory\Available Bytes": definition}, "perfcounters": [{
                "object": "Memory", "counters": [{"name": "Available Bytes"}]
            }]}),
            expected,
        );
    }
}

/// Scenario: Definitions and explicit references use exact local paths, including long and Unicode names.
/// Guarantees: Path names use existing segment and UTF-16 bounds rather than instrument-name restrictions.
#[test]
fn accepts_path_shaped_metric_definitions_and_references() {
    let at_limit = format!(r"\Memory\{}", "a".repeat(MAX_COUNTER_PATH_LEN - 8));
    for name in [
        r"\Memory\Available (Bytes)",
        r"\Process(app#001)\Private Bytes",
        "\\M\u{e9}moire\\Octets disponibles",
        at_limit.as_str(),
    ] {
        let config = RuntimeConfig::from_json(&json!({
            "metrics": {name: {}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": name}]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters[0].name, name);
    }
    let too_long = format!("{at_limit}a");
    let expected = format!("metric name {too_long:?} must contain at most 2047 UTF-16 code units");
    assert_config_error(
        json!({"metrics": {too_long: {}}, "perfcounters": [{
            "object": "Memory", "counters": [{"name": "Available Bytes"}]
        }]}),
        &expected,
    );
}

/// Scenario: Unused path-shaped definitions contain malformed or unsupported path segments.
/// Guarantees: The name accommodation does not bypass existing exact-path safety validation.
#[test]
fn rejects_invalid_path_shaped_metric_definitions() {
    for name in [
        r"\Memory",
        r"\\host\Memory\X",
        r"\Memory(*)\X",
        r"\Memory(app\X",
        r"\Memory()\X",
        r"\Memory(#0)\X",
        r"\Memory\X\Y",
        r"\Memory\X?",
        "\\Memory\\X\0",
    ] {
        assert_config_error(
            json!({"metrics": {name: {}}, "perfcounters": [{
                "object": "Memory", "counters": [{"name": "Available Bytes"}]
            }]}),
            "metric name",
        );
    }
}

/// Scenario: Path-shaped definition names contain padding around object, instance, or counter segments.
/// Guarantees: Definition identities are rejected rather than accepted with padded path segments.
#[test]
fn rejects_padding_in_path_shaped_metric_definitions() {
    for name in [
        r"\Memory \Available Bytes",
        r"\Process( app )\X",
        r"\Memory\ Available Bytes",
    ] {
        assert_config_error(
            json!({"metrics": {name: {}}, "perfcounters": [{
                "object": "Memory", "counters": [{"name": "Available Bytes"}]
            }]}),
            &format!("metric name {name:?} segments must not have surrounding whitespace"),
        );
    }
}

/// Scenario: A counter mapping names a metric that is not defined.
/// Guarantees: Validation identifies the public YAML field that contains the bad reference.
#[test]
fn rejects_undefined_metric_reference() {
    assert_config_error(
        json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "missing"}]
            }]
        }),
        "perfcounters[0].counters[0].metric references undefined metric \"missing\"",
    );
}

/// Scenario: An explicit reference has no definition, or no performance counters are configured.
/// Guarantees: Empty definitions do not satisfy explicit references and counters remain required.
#[test]
fn rejects_undefined_references_and_empty_perfcounters() {
    assert_config_error(
        json!({"metrics": {}, "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available Bytes", "metric": "available"}]
        }]}),
        "references undefined metric \"available\"",
    );
    assert_config_error(
        json!({"metrics": {"available": {
            "description": "Available physical memory.",
            "unit": "By",
            "gauge": {}
        }}, "perfcounters": []}),
        "perfcounters must contain at least one entry",
    );
}

/// Scenario: A metric reference or the definitions map is explicitly null.
/// Guarantees: Null references identify both entry indexes and never select path-name fallback.
#[test]
fn rejects_null_metric_forms() {
    for (object_index, counter_index) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        let mut value = json!({"perfcounters": [
            {"object": "Memory", "counters": [
                {"name": "Available Bytes"}, {"name": "Committed Bytes"}
            ]},
            {"object": "Process", "instances": "worker", "counters": [
                {"name": "Private Bytes"}, {"name": "Working Set"}
            ]}
        ]});
        value["perfcounters"][object_index]["counters"][counter_index]["metric"] = json!(null);
        assert_config_error(
            value,
            &format!(
                "perfcounters[{object_index}].counters[{counter_index}].metric must be a string when present; null metric is invalid"
            ),
        );
    }
    let mut value = config_with_timing("30s", "1s");
    value["metrics"] = json!(null);
    assert_config_error(value, "expected a map");
}

/// Scenario: An object specifies one exact instance as a scalar string.
/// Guarantees: The shorthand normalizes to the same exact instance path as a one-item list.
#[test]
fn normalizes_single_instance_string() {
    let config = RuntimeConfig::from_json(&json!({
        "metrics": {"private": {
            "description": "Committed private memory.",
            "unit": "By",
            "sum": {"aggregation": "cumulative", "monotonic": false}
        }},
        "perfcounters": [{
            "object": "Process",
            "instances": "app",
            "counters": [{"name": "Private Bytes", "metric": "private"}]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters[0].path, r"\Process(app)\Private Bytes");
}

/// Scenario: Instances are configured with a type other than a string or list of strings.
/// Guarantees: The deserialization error names the field and its accepted shapes.
#[test]
fn rejects_invalid_instances_type() {
    assert_config_error(
        gauge_config(json!({
            "object": "Process",
            "instances": 5,
            "counters": [{"name": "Private Bytes", "metric": "available"}]
        })),
        "instances must be a string or a list of strings",
    );
}

/// Scenario: A configured instance uses a wildcard in this exact-path receiver.
/// Guarantees: Validation rejects wildcard expansion until that behavior is supported.
#[test]
fn rejects_wildcard_instances() {
    assert_config_error(
        json!({
            "metrics": metrics(),
            "perfcounters": [{
                "object": "Process",
                "instances": "*",
                "counters": [{"name": "Private Bytes", "metric": "private"}]
            }]
        }),
        "contains a reserved performance-counter path character",
    );
}

/// Scenario: Instance indexes use omitted, zero, or leading-zero spellings.
/// Guarantees: Equivalent PDH instance indexes normalize to one spelling before duplicate checks.
#[test]
fn normalizes_instance_indexes_before_duplicate_detection() {
    for instances in [["app", "app#0"], ["app#1", "app#01"]] {
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [{
                    "object": "Process",
                    "instances": instances,
                    "counters": [{"name": "Private Bytes", "metric": "available"}]
                }]
            }),
            "perfcounters[0].instances contains duplicate",
        );
    }

    let config = RuntimeConfig::from_json(&gauge_config(json!({
        "object": "Process",
        "instances": "app#001",
        "counters": [{"name": "Private Bytes", "metric": "available"}]
    })))
    .unwrap();
    assert_eq!(config.counters[0].path, r"\Process(app#1)\Private Bytes");

    for instance in ["#0", "#1"] {
        assert_config_error(
            gauge_config(json!({
                "object": "Process",
                "instances": instance,
                "counters": [{"name": "Private Bytes", "metric": "available"}]
            })),
            "must include an instance name before its index",
        );
    }
}

/// Scenario: Explicit instance names differ only by letter casing.
/// Guarantees: Validation rejects duplicate instance paths before opening a PDH query.
#[test]
fn rejects_duplicate_instances_case_insensitively() {
    assert_config_error(
        json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [{
                "object": "Process",
                "instances": ["app", "APP"],
                "counters": [{"name": "Private Bytes", "metric": "available"}]
            }]
        }),
        "perfcounters[0].instances contains duplicate \"APP\"",
    );
}

/// Scenario: Object and counter fields contain path delimiters or wildcard characters.
/// Guarantees: Structural characters are rejected while parentheses remain valid in counter names.
#[test]
fn validates_reserved_path_characters_by_field_role() {
    assert_config_error(
        json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [{
                "object": "Memory(test)",
                "counters": [{"name": "Available Bytes", "metric": "available"}]
            }]
        }),
        "perfcounters[0].object contains a reserved",
    );
    assert_config_error(
        json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available *", "metric": "available"}]
            }]
        }),
        "perfcounters[0].counters[0].name contains a reserved",
    );
    let config = RuntimeConfig::from_json(&json!({
        "metrics": {"available": {
            "description": "Available physical memory.",
            "unit": "By",
            "gauge": {}
        }},
        "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available (Bytes)", "metric": "available"}]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters[0].path, r"\Memory\Available (Bytes)");
}

/// Scenario: Object and instance names contain each reserved path character.
/// Guarantees: Every structural, wildcard, or NUL character is rejected in those segments.
#[test]
fn rejects_each_reserved_object_and_instance_character() {
    for reserved in ['\\', '(', ')', '*', '?', '\0'] {
        assert_config_error(
            gauge_config(json!({
                "object": format!("Mem{reserved}ory"),
                "counters": [{"name": "Available Bytes", "metric": "available"}]
            })),
            "perfcounters[0].object contains a reserved",
        );
        assert_config_error(
            gauge_config(json!({
                "object": "Process",
                "instances": format!("app{reserved}"),
                "counters": [{"name": "Private Bytes", "metric": "available"}]
            })),
            "perfcounters[0].instances contains a reserved",
        );
    }
    for reserved in ['\\', '?', '\0'] {
        assert_config_error(
            gauge_config(json!({
                "object": "Memory",
                "counters": [{"name": format!("Available{reserved}"), "metric": "available"}]
            })),
            "perfcounters[0].counters[0].name contains a reserved",
        );
    }
}

/// Scenario: Path segments have leading or trailing whitespace.
/// Guarantees: Segments are trimmed before path construction and duplicate detection.
#[test]
fn trims_surrounding_whitespace_in_path_segments() {
    let config = RuntimeConfig::from_json(&gauge_config(json!({
        "object": " Process ",
        "instances": [" app "],
        "counters": [{"name": " Private Bytes ", "metric": "available"}]
    })))
    .unwrap();
    assert_eq!(config.counters[0].path, r"\Process(app)\Private Bytes");

    assert_config_error(
        gauge_config(json!({
            "object": "Process",
            "instances": ["app", "app "],
            "counters": [{"name": "Private Bytes", "metric": "available"}]
        })),
        "perfcounters[0].instances contains duplicate \"app\"",
    );
    assert_config_error(
        json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [
                {
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "available"}]
                },
                {
                    "object": "Memory ",
                    "counters": [{"name": "Available Bytes", "metric": "available"}]
                }
            ]
        }),
        "duplicate counter path: \\Memory\\Available Bytes",
    );
}

/// Scenario: Counter paths are at and beyond the PDH path limit, including non-BMP text.
/// Guarantees: The limit is enforced in UTF-16 code units, not bytes or chars.
#[test]
fn enforces_counter_path_length_in_utf16_units() {
    // `\Memory\` is 8 code units.
    let at_limit = "a".repeat(MAX_COUNTER_PATH_LEN - 8);
    let _config = RuntimeConfig::from_json(&gauge_config(json!({
        "object": "Memory",
        "counters": [{"name": at_limit, "metric": "available"}]
    })))
    .unwrap();

    let over_limit = "a".repeat(MAX_COUNTER_PATH_LEN - 7);
    let over_limit_path = format!(r"\Memory\{over_limit}");
    let expected =
        format!("counter path {over_limit_path:?} must contain at most 2047 UTF-16 code units");
    assert_config_error(
        gauge_config(json!({
            "object": "Memory",
            "counters": [{"name": over_limit, "metric": "available"}]
        })),
        &expected,
    );

    // `\Process(` + `)\X` is 12 code units; each U+1F600 is 2 code units, 1 char, 4 bytes.
    let at_limit_non_bmp = "\u{1F600}".repeat((MAX_COUNTER_PATH_LEN - 12) / 2);
    let _config = RuntimeConfig::from_json(&gauge_config(json!({
        "object": "Process",
        "instances": at_limit_non_bmp,
        "counters": [{"name": "X", "metric": "available"}]
    })))
    .unwrap();

    let non_bmp = "\u{1F600}".repeat((MAX_COUNTER_PATH_LEN - 12) / 2 + 1);
    assert!(12 + non_bmp.chars().count() <= MAX_COUNTER_PATH_LEN);
    assert_config_error(
        gauge_config(json!({
            "object": "Process",
            "instances": non_bmp,
            "counters": [{"name": "X", "metric": "available"}]
        })),
        "must contain at most 2047 UTF-16 code units",
    );
}

/// Scenario: Object and counter names normalize to the same path with different casing.
/// Guarantees: Validation rejects duplicate native counter handles.
#[test]
fn rejects_duplicate_counter_paths_case_insensitively() {
    assert_config_error(
        json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [
                {
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "available"}]
                },
                {
                    "object": "memory",
                    "counters": [{"name": "available bytes", "metric": "available"}]
                }
            ]
        }),
        "duplicate counter path: \\memory\\available bytes",
    );
}

/// Scenario: Implicit and explicit mappings configure equivalent native paths with different index spellings.
/// Guarantees: Path-derived names do not weaken canonical, case-insensitive native duplicate rejection.
#[test]
fn rejects_duplicate_paths_with_omitted_metric_references() {
    assert_config_error(
        json!({
            "metrics": {"explicit": {}},
            "perfcounters": [
                {"object": "Process", "instances": "app#001", "counters": [{"name": "Private Bytes"}]},
                {"object": "process", "instances": "APP#1", "counters": [{"name": "private bytes", "metric": "explicit"}]}
            ]
        }),
        "duplicate counter path",
    );
}

/// Scenario: Metric names, references, descriptions, units, and attribute keys are padded.
/// Guarantees: Values are trimmed consistently and padded references still resolve.
#[test]
fn trims_metric_definitions_and_attribute_keys() {
    let config = RuntimeConfig::from_json(&json!({
        "metrics": {" available ": {
            "description": " Available memory. ",
            "unit": " By ",
            "gauge": {}
        }},
        "perfcounters": [{
            "object": "Memory",
            "counters": [{
                "name": "Available Bytes",
                "metric": "available ",
                "attributes": {" state ": " free "}
            }]
        }]
    }))
    .unwrap();
    let counter = &config.counters[0];
    assert_eq!(counter.name, "available");
    assert_eq!(&*counter.description, "Available memory.");
    assert_eq!(counter.unit, "By");
    assert_eq!(
        *counter.attributes,
        BTreeMap::from([("state".to_owned(), " free ".to_owned())])
    );
}

/// Scenario: Metric names or attribute keys collide after trimming or case folding.
/// Guarantees: Validation rejects them instead of silently merging definitions.
#[test]
fn rejects_duplicate_metric_names_and_attribute_keys() {
    let metric = json!({"description": "Value.", "unit": "By", "gauge": {}});
    for (first, second) in [("cpu", " cpu"), ("cpu", "CPU")] {
        assert_config_error(
            json!({
                "metrics": {first: metric.clone(), second: metric.clone()},
                "perfcounters": [{
                    "object": "Processor",
                    "counters": [
                        {"name": "% Processor Time", "metric": first},
                        {"name": "% User Time", "metric": second}
                    ]
                }]
            }),
            "metrics contains duplicate",
        );
    }
    assert_config_error(
        gauge_config(json!({
            "object": "Memory",
            "counters": [{
                "name": "Available Bytes",
                "metric": "available",
                "attributes": {"state": "a", " state": "b"}
            }]
        })),
        "contains duplicate attribute key \"state\"",
    );
}

/// Scenario: A configured data-point attribute uses the receiver-owned namespace.
/// Guarantees: Validation preserves ownership of generated performance-counter attributes.
#[test]
fn rejects_reserved_attribute_prefix() {
    assert_config_error(
        json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "available",
                    "attributes": {"windows.perf_counter.custom": "value"}
                }]
            }]
        }),
        "perfcounters[0].counters[0] attribute \"windows.perf_counter.custom\" conflicts",
    );
}

/// Scenario: Decimal scaling is configured at its lower and upper bounds.
/// Guarantees: Inclusive boundary values are accepted and retained.
#[test]
fn accepts_scaling_bounds() {
    for scale_power10 in [MIN_SCALE_POWER10, MAX_SCALE_POWER10] {
        let config = RuntimeConfig::from_json(&gauge_config(json!({
            "object": "Memory",
            "counters": [{
                "name": "Available Bytes",
                "metric": "available",
                "scale_power10": scale_power10
            }]
        })))
        .unwrap();
        assert_eq!(config.counters[0].scale_power10, scale_power10);
    }
}

/// Scenario: Decimal scaling exceeds the receiver's exact integer-power range.
/// Guarantees: Validation rejects both lower and upper out-of-range values with the path.
#[test]
fn rejects_out_of_range_scaling() {
    for scale_power10 in [-19, 19] {
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{
                        "name": "Available Bytes",
                        "metric": "available",
                        "scale_power10": scale_power10
                    }]
                }]
            }),
            "counter path \"\\\\Memory\\\\Available Bytes\" scale_power10",
        );
    }
}

/// Scenario: Instance and counter lists multiply into many expanded counter paths.
/// Guarantees: The expanded total is bounded before any paths are materialized.
#[test]
fn bounds_expanded_counter_paths() {
    let object = |instances: usize, counters: usize| {
        json!({
            "object": "Process",
            "instances": (0..instances).map(|i| format!("i{i}")).collect::<Vec<_>>(),
            "counters": (0..counters)
                .map(|c| json!({"name": format!("C{c}"), "metric": "available"}))
                .collect::<Vec<_>>()
        })
    };

    let config = RuntimeConfig::from_json(&gauge_config(object(16, 16))).unwrap();
    assert_eq!(config.counters.len(), MAX_COUNTERS);

    assert_config_error(
        gauge_config(object(17, 16)),
        "perfcounters must expand to at most 256 counter paths",
    );
    assert_config_error(
        gauge_config(object(10_000, 10_000)),
        "perfcounters must expand to at most 256 counter paths",
    );
}

/// Scenario: Several object entries each stay under the cap but exceed it together.
/// Guarantees: The expanded-path bound applies to the total across all entries.
#[test]
fn bounds_expanded_counter_paths_across_objects() {
    let objects = |count: usize| {
        json!({
            "metrics": {"available": {"description": "Value.", "unit": "By", "gauge": {}}},
            "perfcounters": (0..2)
                .map(|o| json!({
                    "object": format!("Object{o}"),
                    "counters": (0..count)
                        .map(|c| json!({"name": format!("C{c}"), "metric": "available"}))
                        .collect::<Vec<_>>()
                }))
                .collect::<Vec<_>>()
        })
    };
    let config = RuntimeConfig::from_json(&objects(MAX_COUNTERS / 2)).unwrap();
    assert_eq!(config.counters.len(), MAX_COUNTERS);
    assert_config_error(
        objects(MAX_COUNTERS / 2 + 1),
        "perfcounters must expand to at most 256 counter paths",
    );
}
