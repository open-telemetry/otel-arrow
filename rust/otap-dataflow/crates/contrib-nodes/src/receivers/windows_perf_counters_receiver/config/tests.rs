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

/// Scenario: Exact object counters use no instance or explicitly named instances.
/// Guarantees: Structured configuration normalizes paths and retains both metric kinds.
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
                "counters": [{"name": "Private Bytes", "metric": "private"}]
            }
        ]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 3);
    assert_eq!(config.counters[0].path, r"\Memory\Available Bytes");
    assert_eq!(config.counters[0].metric_kind, MetricKind::Gauge);
    assert_eq!(config.counters[1].path, r"\Process(app)\Private Bytes");
    assert_eq!(config.counters[1].metric_kind, MetricKind::Sum);
    assert_eq!(config.collection_interval, Duration::from_secs(60));
    assert_eq!(config.initial_delay, Duration::from_secs(1));
}

/// Scenario: Collection timing uses sub-second and multi-day durations or a zero collection interval.
/// Guarantees: Positive collection intervals and nonnegative delays are unbounded; zero collection identifies its field.
#[test]
fn enforces_collection_timing_bounds() {
    let _config = Config::from_json(&config_with_timing("24h", "24h")).unwrap();

    for (interval, delay) in [("500ms", "0s"), ("86401s", "0s"), ("1s", "86401s")] {
        let _config = Config::from_json(&config_with_timing(interval, delay)).unwrap();
    }
    assert_config_error(
        config_with_timing("0s", "0s"),
        "collection_interval must be greater than zero",
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
/// Guarantees: Empty counter lists remain invalid; an empty instance list selects the object's non-instanced counter.
#[test]
fn rejects_empty_object_collections() {
    assert_config_error(
        gauge_config(json!({"object": "Memory", "counters": []})),
        "perfcounters[0].counters must contain at least one entry",
    );
    let config = Config::from_json(&gauge_config(json!({
        "object": "Process",
        "instances": [],
        "counters": [{"name": "Private Bytes", "metric": "available"}]
    })))
    .unwrap();
    assert_eq!(config.counters.len(), 1);
    assert_eq!(config.counters[0].path, r"\Process\Private Bytes");
}

/// Scenario: A metric definition name or an attribute key is literally empty.
/// Guarantees: Empty names and keys remain configuration errors with field context.
#[test]
fn rejects_empty_required_values() {
    assert_config_error(
        json!({
            "metrics": {"": {"description": "Value.", "unit": "By", "gauge": {}}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": ""}]
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
                "attributes": {"": "value"}
            }]
        })),
        "perfcounters[0].counters[0] attribute key must not be empty",
    );
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
        let config = Config::from_json(&value).unwrap();
        assert_eq!(config.counters[0].metric_kind, MetricKind::Sum);
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
        value["metrics"] = json!({"value": {
            "description": "Value.", "unit": "1", "gauge": gauge
        }});
        value["perfcounters"][0]["counters"][0]["metric"] = json!("value");
        let config = Config::from_json(&value).unwrap();
        assert_eq!(config.counters[0].metric_kind, MetricKind::Gauge);
        value["metrics"]["value"]["sum"] = json!({"aggregation": "cumulative"});
        assert_eq!(
            Config::from_json(&value).unwrap_err().to_string(),
            "An invalid user configuration occurred: metrics.value must not contain both gauge and sum"
        );
    }
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
        let error = Config::from_json(&value).unwrap_err().to_string();
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

/// Scenario: Referenced Gauge metadata is omitted, empty, whitespace-only or explicit, and another definition is unused.
/// Guarantees: Missing kind/metadata receive defaults without trimming supplied text, rejecting unused definitions or changing timing.
#[test]
fn defaults_optional_metric_metadata_and_allows_unused_definitions() {
    for description in [
        None,
        Some(""),
        Some(" \t "),
        Some("Available physical memory."),
    ] {
        for unit in [None, Some(""), Some(" \t "), Some("By")] {
            for explicit_gauge in [false, true] {
                let mut definition = json!({});
                if let Some(description) = description {
                    definition["description"] = json!(description);
                }
                if let Some(unit) = unit {
                    definition["unit"] = json!(unit);
                }
                if explicit_gauge {
                    definition["gauge"] = json!({});
                }
                let config = Config::from_json(&json!({
                    "metrics": {"used": definition, "unused": {}},
                    "perfcounters": [{
                        "object": "Memory",
                        "counters": [{"name": "Available Bytes", "metric": "used"}]
                    }]
                }))
                .unwrap();
                assert_eq!(config.counters.len(), 1);
                let counter = &config.counters[0];
                assert_eq!(counter.name.as_ref(), "used");
                assert_eq!(counter.description.as_ref(), description.unwrap_or(""));
                assert_eq!(
                    counter.unit.as_ref(),
                    unit.filter(|unit| !unit.is_empty()).unwrap_or("1")
                );
                assert_eq!(counter.metric_kind, MetricKind::Gauge);
                assert_eq!(config.collection_interval, Duration::from_secs(60));
                assert_eq!(config.initial_delay, Duration::from_secs(1));
            }
        }
    }
}

/// Scenario: OTLP metric names do not satisfy SDK instrument-name syntax or length limits.
/// Guarantees: Nonempty whitespace-only, Unicode, numeric, punctuated and long names resolve unchanged.
#[test]
fn accepts_unrestricted_metric_names() {
    let long_name = format!("metric {}", "x".repeat(512));
    for name in [
        "  ",
        "1available",
        "avail able",
        "\u{e9}",
        "metric:!{}",
        "system.memory_usage-1/s",
        long_name.as_str(),
    ] {
        let config = Config::from_json(&json!({
            "metrics": {name: {"unit": "By"}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": name}]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters[0].name.as_ref(), name);
        assert_eq!(config.counters[0].unit.as_ref(), "By");
    }
}

/// Scenario: Metric units contain Unicode or long arbitrary text.
/// Guarantees: Every supplied nonempty unit is retained verbatim.
#[test]
fn accepts_unrestricted_metric_units() {
    let unit_config = |unit: &str| {
        json!({
            "metrics": {"m": {"description": "Value.", "unit": unit, "gauge": {}}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "m"}]
            }]
        })
    };
    let long_unit = "a".repeat(128);
    for unit in [
        " ",
        " By ",
        "By",
        "{packets}/s",
        "1",
        "\u{b5}s",
        "B y",
        long_unit.as_str(),
    ] {
        let config = Config::from_json(&unit_config(unit)).unwrap();
        assert_eq!(config.counters[0].unit.as_ref(), unit);
    }
}

/// Scenario: A metric name, its exact reference and its unit contain control characters.
/// Guarantees: Metric names, references and units retain their configured control characters.
/// Metric metadata is not a native PDH selector and must not inherit native-component restrictions.
#[test]
fn preserves_control_characters_in_metric_names_references_and_units() {
    let name = "metric\nname";
    let unit = "B\0y";
    let config = Config::from_json(&json!({
        "metrics": {name: {"unit": unit}},
        "perfcounters": [{"object": "Memory", "counters": [
            {"name": "Available Bytes", "metric": name}
        ]}]
    }))
    .unwrap();
    assert_eq!(config.counters[0].name.as_ref(), name);
    assert_eq!(config.counters[0].unit.as_ref(), unit);
}

/// Scenario: A metric description is configured using a YAML literal block.
/// Guarantees: The block's trailing newline reaches the metric description unchanged.
/// Users control block chomping in YAML; the receiver must not trim the parsed string.
#[test]
fn preserves_yaml_block_description() {
    let value: serde_json::Value = serde_yaml::from_str(
        r#"
metrics:
  available:
    description: |
      Available memory.
perfcounters:
  - object: Memory
    counters:
      - name: Available Bytes
        metric: available
"#,
    )
    .unwrap();
    let config = Config::from_json(&value).unwrap();
    assert_eq!(
        config.counters[0].description.as_ref(),
        "Available memory.\n"
    );
}

/// Scenario: Definitions exceed the configured native-path budget but only one is referenced.
/// Guarantees: Unused definitions do not consume the counter-path budget or produce counters.
#[test]
fn accepts_metric_definitions_beyond_path_limit() {
    let metrics = (0..=DEFAULT_MAX_EXPANDED_COUNTERS)
        .map(|index| {
            (
                format!("m{index}"),
                json!({"description": "Value.", "unit": "By", "gauge": {}}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let config = Config::from_json(&json!({
        "metrics": metrics,
        "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available Bytes", "metric": "m0"}]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 1);
    assert_eq!(config.counters[0].name.as_ref(), "m0");
}

/// Scenario: An unused definition selects an unsupported metric kind.
/// Guarantees: All definitions are validated even when no mapping references them.
#[test]
fn validates_unused_metric_definitions() {
    let mut value = config_with_timing("30s", "1s");
    value["metrics"]["unused"] = json!({"sum": {"aggregation": "delta"}});
    assert_config_error(value, "metrics.unused: delta Sum is not supported yet");
}

/// Scenario: Definitions are omitted or empty and metric references are omitted or empty.
/// Guarantees: Path-named Gauges retain the same configured spelling as native paths.
#[test]
fn defaults_path_metrics_without_definitions() {
    for definitions in [None, Some(json!({}))] {
        let mut value = json!({"perfcounters": [{
            "object": "Process", "instances": ["app#00", "worker#001"],
            "counters": [{"name": "Private Bytes", "scale_power10": -3}]
        }]});
        if let Some(definitions) = definitions {
            value["metrics"] = definitions;
        }
        for reference in [None, Some("")] {
            if let Some(reference) = reference {
                value["perfcounters"][0]["counters"][0]["metric"] = json!(reference);
            }
            let config = Config::from_json(&value).unwrap();
            for (counter, expected_path) in [
                (&config.counters[0], r"\Process(app#00)\Private Bytes"),
                (&config.counters[1], r"\Process(worker#001)\Private Bytes"),
            ] {
                assert_eq!(counter.name.as_ref(), expected_path);
                assert_eq!(counter.path, expected_path);
                assert_eq!(counter.metric_kind, MetricKind::Gauge);
                assert_eq!(counter.unit.as_ref(), "1");
                assert!(counter.description.is_empty());
                assert!(counter.attributes.is_empty());
                assert_eq!(counter.scale_power10, -3);
            }
        }
    }
}

/// Scenario: An implicitly matched path definition selects conflicting or unsupported kinds.
/// Guarantees: Path-name resolution never replaces an invalid definition with the default Gauge.
#[test]
fn validates_implicit_path_metric_definitions() {
    for (definition, expected) in [
        (
            json!({"sum": {"aggregation": "delta"}}),
            "delta Sum is not supported",
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
/// Guarantees: References resolve literal definition keys. Path-shaped names are metric names,
/// so their spelling and length aren't checked as native paths.
#[test]
fn accepts_path_shaped_metric_definitions_and_references() {
    let overlong = format!(r"\Memory\{}", "\u{1f600}".repeat(MAX_COUNTER_PATH_LEN));
    for name in [
        r"\Memory\Available (Bytes)",
        r"\Process(app#001)\Private Bytes",
        r"\Hyper-V Virtual Switch Port(*)\IPsec SAs Offloaded ",
        "\\M\u{e9}moire\\Octets disponibles",
        overlong.as_str(),
    ] {
        let config = Config::from_json(&json!({
            "metrics": {name: {}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": name}]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters[0].name.as_ref(), name);
    }
}

/// Scenario: Path-shaped definitions use opaque names or differ from native paths by trailing spaces.
/// Guarantees: Implicit lookup requires exact configured spelling; explicit references select literal definitions.
/// Padding changes metric identity; only generated paths are sent to PDH.
#[test]
fn matches_path_definitions_only_by_exact_spelling() {
    for (name, counter) in [
        ("\\Memory\\Available Bytes ", "Available Bytes"),
        (r"\Memory", "Available Bytes"),
        ("\\Memory\\Available Bytes ", "Available Bytes "),
    ] {
        let path = counter_path("Memory", None, counter);
        let mut value = json!({"metrics": {name: {"unit": "exact"}}, "perfcounters": [{
            "object": "Memory", "counters": [{"name": counter}]
        }]});
        let config = Config::from_json(&value).unwrap();
        assert_eq!(config.counters[0].name.as_ref(), path);
        assert_eq!(
            config.counters[0].unit.as_ref(),
            if name == path { "exact" } else { "1" }
        );
        value["perfcounters"][0]["counters"][0]["metric"] = json!(name);
        let config = Config::from_json(&value).unwrap();
        assert_eq!(config.counters[0].name.as_ref(), name);
        assert_eq!(config.counters[0].unit.as_ref(), "exact");
    }
}

/// Scenario: Definitions include a leading-whitespace name, an explicit alias, an unavailable implicit path, an unselected case variant and an overlong name.
/// Guarantees: Only path-shaped definitions no counter selects warn, whatever their length or availability; keys with leading whitespace are ordinary names and never warn.
/// A valid alias or unavailable counter must not be misdiagnosed as an unused definition.
#[test]
fn warns_only_for_unselected_path_metric_definitions() {
    let overlong = format!(r"\Memory\{}", "\u{1f600}".repeat(MAX_COUNTER_PATH_LEN / 2));
    let names = [
        " \\NotPath\\Unused ",
        r"\Alias*?",
        r"\Unavailable\Counter",
        r"\unavailable\counter",
        overlong.as_str(),
    ];
    let definitions = names
        .iter()
        .map(|name| ((*name).to_owned(), json!({"unit": "selected"})))
        .collect::<serde_json::Map<_, _>>();
    let mut value = json!({
        "metrics": definitions,
        "perfcounters": [
            {"object": "Memory", "counters": [
                {"name": "Available Bytes", "metric": r"\Alias*?"}
            ]},
            {"object": "Unavailable", "counters": [{"name": "Counter"}]}
        ]
    });
    let config = Config::from_json(&value).unwrap();
    assert!(
        config
            .counters
            .iter()
            .all(|counter| counter.unit.as_ref() == "selected")
    );
    assert_eq!(
        unused_path_metric_definitions(
            value["metrics"]
                .as_object()
                .unwrap()
                .keys()
                .map(|name| (name.as_str(), is_path_metric_name(name))),
            config.counters.iter().map(|counter| counter.name.as_ref()),
        ),
        [overlong.as_str(), r"\unavailable\counter",]
    );
    value["perfcounters"][0]["counters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "Committed Bytes", "metric": overlong}));
    let config = Config::from_json(&value).unwrap();
    assert_eq!(
        unused_path_metric_definitions(
            value["metrics"]
                .as_object()
                .unwrap()
                .keys()
                .map(|name| (name.as_str(), is_path_metric_name(name))),
            config.counters.iter().map(|counter| counter.name.as_ref()),
        ),
        [r"\unavailable\counter"]
    );
}

/// Scenario: Definitions include an ASCII case group, a Unicode pair, a unique name and a whitespace-different name.
/// Guarantees: Only case-distinct groups warn; whitespace-different names stay separate.
/// A case-only difference is likely a typo, but all names are valid, so this warns rather than rejects.
#[test]
fn warns_once_per_case_distinct_group() {
    let names = ["cpu", "CPU", "\u{c9}", "\u{e9}", "unique", " cpu"];
    assert_eq!(
        case_distinct_metric_definitions(names)
            .into_values()
            .collect::<Vec<_>>(),
        [vec!["cpu", "CPU"], vec!["\u{c9}", "\u{e9}"],]
    );
}

/// Scenario: Case-distinct ordinary and path definitions are referenced explicitly and by path fallback.
/// Guarantees: Lookup remains exact and case-sensitive, preserving separate metadata and defaulting mismatched paths.
/// OTLP metric names are case-sensitive.
#[test]
fn resolves_case_distinct_definitions_exactly() {
    let mut value = json!({
        "metrics": {
            "cpu": {"unit": "lower"},
            "CPU": {"unit": "upper"},
            r"\Memory\Available Bytes": {"unit": "By"},
            r"\memory\available bytes": {"unit": "\u{b5}s", "sum": {"aggregation": "cumulative"}}
        },
        "perfcounters": [
            {"object": "Processor", "counters": [
                {"name": "Time", "metric": "cpu"},
                {"name": "User Time", "metric": "CPU"}
            ]},
            {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
            {"object": "memory", "counters": [{"name": "committed bytes",
                "metric": r"\memory\available bytes"}]},
            {"object": "MEMORY", "counters": [{"name": "CACHE BYTES"}]}
        ]
    });
    let config = Config::from_json(&value).unwrap();
    for (counter, name, unit, kind) in [
        (&config.counters[0], "cpu", "lower", MetricKind::Gauge),
        (&config.counters[1], "CPU", "upper", MetricKind::Gauge),
        (
            &config.counters[2],
            r"\Memory\Available Bytes",
            "By",
            MetricKind::Gauge,
        ),
        (
            &config.counters[3],
            r"\memory\available bytes",
            "\u{b5}s",
            MetricKind::Sum,
        ),
        (
            &config.counters[4],
            r"\MEMORY\CACHE BYTES",
            "1",
            MetricKind::Gauge,
        ),
    ] {
        assert_eq!(counter.name.as_ref(), name);
        assert_eq!(counter.unit.as_ref(), unit);
        assert_eq!(counter.metric_kind, kind);
    }
    value["perfcounters"][0]["counters"][0]["metric"] = json!("Cpu");
    assert_config_error(value, "references undefined metric \"Cpu\"");
}

/// Scenario: Implicit and explicit mappings configure numeric-suffix aliases with different casing.
/// Guarantees: Alias and case-insensitive duplicate rejection applies regardless of metric naming.
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

/// Scenario: Equivalent instance suffixes are configured in separate object entries.
/// Guarantees: Zero and leading-zero aliases cannot bypass case-insensitive duplicate-path rejection.
#[test]
fn rejects_instance_aliases_across_object_entries() {
    for [first, second] in [
        ["app", "app#0"],
        ["app#1", "app#01"],
        ["App", "app#00"],
        ["\u{00c4}pp#1", "\u{00e4}pp#001"],
    ] {
        assert_config_error(
            json!({"perfcounters": [
                {"object": "Process", "instances": first, "counters": [{"name": "Private Bytes"}]},
                {"object": "process", "instances": second, "counters": [{"name": "private bytes"}]}
            ]}),
            "duplicate counter path",
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

/// Scenario: No performance counters are configured.
/// Guarantees: Metric definitions cannot satisfy the required nonempty counter configuration.
#[test]
fn rejects_empty_perfcounters() {
    assert_config_error(
        json!({"metrics": {"available": {
            "description": "Available physical memory.",
            "unit": "By",
            "gauge": {}
        }}, "perfcounters": []}),
        "perfcounters must contain at least one entry",
    );
}

/// Scenario: A metric reference is explicitly null at different object and counter indexes.
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
}

/// Scenario: The metric definitions map is explicitly null and a counter uses implicit naming.
/// Guarantees: A null definitions map behaves like an empty map and produces a path-named Gauge.
/// Go accepts null metric maps; no definitions are required for implicit collection.
#[test]
fn accepts_null_metric_definitions_with_implicit_names() {
    let config = Config::from_json(&json!({
        "metrics": null,
        "perfcounters": [{"object": "Memory", "counters": [{"name": "Available Bytes"}]}]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 1);
    let counter = &config.counters[0];
    assert_eq!(counter.name.as_ref(), r"\Memory\Available Bytes");
    assert_eq!(counter.unit.as_ref(), "1");
    assert!(counter.description.is_empty());
    assert_eq!(counter.metric_kind, MetricKind::Gauge);
}

/// Scenario: An object specifies one exact instance as a scalar string.
/// Guarantees: The shorthand normalizes to the same exact instance path as a one-item list.
#[test]
fn normalizes_single_instance_string() {
    let config = Config::from_json(&json!({
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

/// Scenario: An object, instance or counter component contains a literal wildcard.
/// Guarantees: Only instance operators select discovery; other components remain verbatim native selectors, not config-layer expansions.
#[test]
fn normalizes_instance_wildcards_and_preserves_other_components() {
    for pattern in ["*", "?"] {
        for (object, instance, counter, wildcard) in [
            (pattern, "app", "Value", false),
            ("Process", pattern, "Value", true),
            ("Process", "app", pattern, false),
        ] {
            let config = Config::from_json(&json!({"perfcounters": [{
                "object": object, "instances": instance, "counters": [{"name": counter}]
            }]}))
            .unwrap();
            assert_eq!(config.counters.len(), 1);
            assert_eq!(
                config.counters[0].path,
                format!(r"\{object}({instance})\{counter}")
            );
            assert_eq!(config.counters[0].instance_config.is_wildcard(), wildcard);
        }
    }
}

/// Scenario: Instance indexes and an excluded aggregation use omitted, zero or leading-zero spellings.
/// Guarantees: Native paths, implicit lookup and aggregation exclusion preserve each configured spelling.
#[test]
fn preserves_instance_indexes_in_paths_and_lookup() {
    for (instance, unit) in [
        ("app", "By"),
        ("app#0", "KiBy"),
        ("app#00", "MiBy"),
        ("app#1", "GiBy"),
        ("app#01", "TiBy"),
        ("app#001", "PiBy"),
        ("_Total#00", "By"),
    ] {
        let config = Config::from_json(&json!({
            "metrics": {
                r"\Process(app)\Private Bytes": {"unit": "By"},
                r"\Process(app#0)\Private Bytes": {"unit": "KiBy"},
                r"\Process(app#00)\Private Bytes": {"unit": "MiBy"},
                r"\Process(app#1)\Private Bytes": {"unit": "GiBy"},
                r"\Process(app#01)\Private Bytes": {"unit": "TiBy"},
                r"\Process(app#001)\Private Bytes": {"unit": "PiBy"},
                r"\Process(_Total#00)\Private Bytes": {"unit": "By"}
            },
            "perfcounters": [{
                "object": "Process",
                "instances": instance,
                "counters": [{"name": "Private Bytes"}]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters.len(), 1);
        let counter = &config.counters[0];
        assert_eq!(counter.path, format!(r"\Process({instance})\Private Bytes"));
        assert_eq!(counter.name.as_ref(), counter.path);
        assert_eq!(counter.unit.as_ref(), unit);
    }

    // Keep the existing exact-only grammar restriction.
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
    let wildcard = Config::from_json(&json!({"perfcounters": [{
        "object": "Process",
        "instances": "worker*",
        "aggregation_name": "_Total#00",
        "counters": [{"name": "Private Bytes"}]
    }]}))
    .unwrap();
    assert_eq!(
        wildcard.counters[0].instance_config.excluded_aggregation(),
        Some("_Total#00")
    );
}

/// Scenario: A single instance list contains case-insensitive numeric-suffix aliases.
/// Guarantees: Omitted, zero and leading-zero forms cannot create duplicate native observations.
#[test]
fn rejects_instance_aliases_in_one_list() {
    for instances in [
        ["app", "app#0"],
        ["app#1", "app#01"],
        ["App", "app#00"],
        ["\u{00c4}pp#1", "\u{00e4}pp#001"],
    ] {
        assert_config_error(
            json!({"perfcounters": [{
                "object": "Process",
                "instances": instances,
                "counters": [{"name": "Private Bytes"}]
            }]}),
            "perfcounters[0].instances contains duplicate",
        );
    }
}

/// Scenario: `app`, `app#1` and `app#2` coexist with suffix-like literal instance text.
/// Guarantees: The three indexes stay distinct; `app#2` guards against dropping index digits.
/// `app#0 ` and ` app#0` retain spaces; `app#01 ` and `app#01x` do not collide with `app#1`.
/// `app#1x` is nonnumeric, `app#\u{0660}` is not ASCII numeric, and `app#0#0` is not normalized recursively.
#[test]
fn preserves_distinct_instance_suffix_text() {
    let instances = [
        "app",
        "app#1",
        "app#2",
        "app#0 ",
        "app#01 ",
        " app#0",
        "app#1x",
        "app#01x",
        "app#\u{0660}",
        "app#0#0",
    ];
    let config = Config::from_json(&json!({"perfcounters": [{
        "object": "Process",
        "instances": instances,
        "counters": [{"name": "Private Bytes"}]
    }]}))
    .unwrap();
    assert_eq!(config.counters.len(), instances.len());
    for (counter, instance) in config.counters.iter().zip(instances) {
        assert_eq!(counter.path, format!(r"\Process({instance})\Private Bytes"));
        assert_eq!(counter.name.as_ref(), counter.path);
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

/// Scenario: Object, instance and counter components contain delimiters, wildcard characters or NUL.
/// Guarantees: NUL is rejected before FFI; other native spelling is preserved without a substitute PDH grammar.
#[test]
fn preserves_non_nul_reserved_components_and_rejects_nul() {
    for reserved in ['\\', '(', ')', '*', '?', '\0'] {
        let object = gauge_config(json!({
            "object": format!("Mem{reserved}ory"),
            "counters": [{"name": "Available Bytes", "metric": "available"}]
        }));
        let instance = gauge_config(json!({
            "object": "Process",
            "instances": format!("app{reserved}"),
            "counters": [{"name": "Private Bytes", "metric": "available"}]
        }));
        if reserved == '\0' {
            assert_config_error(object, "perfcounters[0].object must not contain NUL");
            assert_config_error(instance, "perfcounters[0].instances must not contain NUL");
        } else {
            let object = Config::from_json(&object).unwrap();
            assert_eq!(
                object.counters[0].path,
                format!(r"\Mem{reserved}ory\Available Bytes")
            );
            let instance = Config::from_json(&instance).unwrap();
            assert_eq!(
                instance.counters[0].path,
                format!(r"\Process(app{reserved})\Private Bytes")
            );
            assert_eq!(
                instance.counters[0].instance_config.is_wildcard(),
                matches!(reserved, '*' | '?')
            );
        }
    }
    for reserved in ['\\', '(', ')', '*', '?', '\0'] {
        let counter = gauge_config(json!({
            "object": "Memory",
            "counters": [{"name": format!("Available{reserved}"), "metric": "available"}]
        }));
        if reserved == '\0' {
            assert_config_error(
                counter,
                "perfcounters[0].counters[0].name must not contain NUL",
            );
        } else {
            let counter = Config::from_json(&counter).unwrap();
            assert_eq!(
                counter.counters[0].path,
                format!(r"\Memory\Available{reserved}")
            );
            assert!(!counter.counters[0].instance_config.is_wildcard());
        }
    }
}

/// Scenario: Path segments have leading or trailing whitespace.
/// Guarantees: Padding is retained in paths and remains distinct during duplicate detection.
#[test]
fn preserves_surrounding_whitespace_in_path_segments() {
    let config = Config::from_json(&gauge_config(json!({
        "object": " Process ",
        "instances": [" app "],
        "counters": [{"name": " Private Bytes ", "metric": "available"}]
    })))
    .unwrap();
    assert_eq!(
        config.counters[0].path,
        r"\ Process ( app )\ Private Bytes "
    );

    let config = Config::from_json(&gauge_config(json!({
        "object": "Process",
        "instances": ["app", "app "],
        "counters": [{"name": "Private Bytes", "metric": "available"}]
    })))
    .unwrap();
    assert_eq!(config.counters[0].path, r"\Process(app)\Private Bytes");
    assert_eq!(config.counters[1].path, r"\Process(app )\Private Bytes");
    let config = Config::from_json(&json!({
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
    }))
    .unwrap();
    assert_eq!(config.counters[0].path, r"\Memory\Available Bytes");
    assert_eq!(config.counters[1].path, r"\Memory \Available Bytes");
}

/// Scenario: Native components contain only whitespace, non-NUL controls, or literal empty text.
/// Guarantees: Nonempty spelling is preserved while literal empties remain configuration errors.
#[test]
fn preserves_nonempty_native_component_text() {
    for (object, instance, counter) in [
        (" ", "\t", "\r\n"),
        ("Mem\tory", "work\ner", "Private\rBytes"),
    ] {
        let config = Config::from_json(&json!({"perfcounters": [{
            "object": object, "instances": instance, "counters": [{"name": counter}]
        }]}))
        .unwrap();
        let path = format!("\\{object}({instance})\\{counter}");
        assert_eq!(config.counters[0].path, path);
        assert_eq!(config.counters[0].name.as_ref(), path);
    }
    for (object, instance, counter, field) in [
        ("", "app", "Value", "perfcounters[0].object"),
        ("Process", "", "Value", "perfcounters[0].instances"),
        ("Process", "app", "", "perfcounters[0].counters[0].name"),
    ] {
        assert_config_error(
            json!({"perfcounters": [{
                "object": object, "instances": instance, "counters": [{"name": counter}]
            }]}),
            &format!("{field} must not be empty"),
        );
    }
}

/// Scenario: Counter paths are at and beyond the PDH path limit, including non-BMP text.
/// Guarantees: The limit is enforced in UTF-16 code units, not bytes or chars.
#[test]
fn enforces_counter_path_length_in_utf16_units() {
    // `\Memory\` is 8 code units.
    let at_limit = "a".repeat(MAX_COUNTER_PATH_LEN - 8);
    let _config = Config::from_json(&gauge_config(json!({
        "object": "Memory",
        "counters": [{"name": &at_limit, "metric": "available"}]
    })))
    .unwrap();
    assert_config_error(
        gauge_config(json!({
            "object": "Memory",
            "counters": [{"name": format!("{at_limit} "), "metric": "available"}]
        })),
        "must contain at most 2047 UTF-16 code units",
    );

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
    let _config = Config::from_json(&gauge_config(json!({
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

/// Scenario: Metric names, references, descriptions, units, and attribute keys are padded.
/// Guarantees: Metric metadata, attribute keys, and string values reach CounterConfig unchanged.
/// The receiver must not rewrite user-supplied metric metadata.
#[test]
fn preserves_metric_identities_and_metadata() {
    let config = Config::from_json(&json!({
        "metrics": {" available ": {
            "description": " Available\tmemory. ",
            "unit": "\r By \t",
            "gauge": {}
        }},
        "perfcounters": [{
            "object": "Memory",
            "counters": [{
                "name": "Available Bytes",
                "metric": " available ",
                "attributes": {" host ": " free "}
            }]
        }]
    }))
    .unwrap();
    let counter = &config.counters[0];
    assert_eq!(counter.name.as_ref(), " available ");
    assert_eq!(&*counter.description, " Available\tmemory. ");
    assert_eq!(counter.unit.as_ref(), "\r By \t");
    assert_eq!(
        *counter.attributes,
        BTreeMap::from([(" host ".to_owned(), " free ".to_owned())])
    );
}

/// Scenario: Metric names differ only by leading or trailing spaces.
/// Guarantees: Both metrics load with their own metadata without merging.
/// Distinct quoted identities must not collide because their unquoted text looks alike.
#[test]
fn allows_whitespace_distinct_metric_names() {
    for (first, second) in [("foo", " foo"), ("foo", "foo ")] {
        let config = Config::from_json(&json!({
            "metrics": {first: {"unit": "By"}, second: {"unit": "KiBy"}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [
                    {"name": "Available Bytes", "metric": first},
                    {"name": "Committed Bytes", "metric": second}
                ]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters.len(), 2);
        for (counter, name, unit) in [
            (&config.counters[0], first, "By"),
            (&config.counters[1], second, "KiBy"),
        ] {
            assert_eq!(counter.name.as_ref(), name);
            assert_eq!(counter.unit.as_ref(), unit);
        }
    }
}

/// Scenario: Explicit metric references are empty, padded, whitespace-only or missing their exact definition.
/// Guarantees: Only empty references use the implicit path; nonempty references select their exact definition or fail as undefined.
/// A mistyped reference must not silently select a different metric or the implicit path.
#[test]
fn resolves_metric_references_exactly() {
    let path = r"\Memory\Available Bytes";
    let mut value = json!({
        "metrics": {
            "foo": {"unit": "plain"},
            " foo": {"unit": "padded"},
            " \t ": {"unit": "whitespace"},
            path: {"unit": "path"}
        },
        "perfcounters": [{"object": "Memory", "counters": [
            {"name": "Available Bytes", "metric": ""}
        ]}]
    });
    for (reference, name, unit) in [
        ("", path, "path"),
        ("foo", "foo", "plain"),
        (" foo", " foo", "padded"),
        (" \t ", " \t ", "whitespace"),
    ] {
        value["perfcounters"][0]["counters"][0]["metric"] = json!(reference);
        let config = Config::from_json(&value).unwrap();
        assert_eq!(config.counters[0].name.as_ref(), name);
        assert_eq!(config.counters[0].unit.as_ref(), unit);
    }
    for reference in ["  ", "foo ", " foo "] {
        value["perfcounters"][0]["counters"][0]["metric"] = json!(reference);
        assert_config_error(
            value.clone(),
            &format!("references undefined metric {reference:?}"),
        );
    }
    let _ = value["metrics"].as_object_mut().unwrap().remove(" foo");
    value["perfcounters"][0]["counters"][0]["metric"] = json!(" foo");
    assert_config_error(value, "references undefined metric \" foo\"");
}

/// Scenario: A configured data-point attribute uses the receiver-owned namespace.
/// Guarantees: Validation preserves ownership of generated performance-counter attributes.
#[test]
fn rejects_reserved_attribute_prefix() {
    for key in [
        "windows.perf_counter.",
        "windows.perf_counter.custom",
        "windows.perf_counter.path ",
        "windows.perf_counter.\0",
    ] {
        assert_config_error(
            gauge_config(json!({
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "available",
                    "attributes": {key: "value"}
                }]
            })),
            &format!("perfcounters[0].counters[0] attribute {key:?} conflicts"),
        );
    }
}

/// Scenario: Decimal scaling is configured at its lower and upper bounds.
/// Guarantees: Inclusive boundary values are accepted and retained.
#[test]
fn accepts_scaling_bounds() {
    for scale_power10 in [MIN_SCALE_POWER10, MAX_SCALE_POWER10] {
        let config = Config::from_json(&gauge_config(json!({
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
/// Guarantees: The configured selector budget is enforced incrementally without materializing the full product.
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

    let limited = |object| {
        let mut config = gauge_config(object);
        config["max_expanded_counters"] = json!(256);
        config
    };
    let config = Config::from_json(&limited(object(16, 16))).unwrap();
    assert_eq!(config.counters.len(), 256);

    assert_config_error(
        limited(object(17, 16)),
        "exceeding max_expanded_counters 256",
    );
}

/// Scenario: Several object entries each stay under the cap but exceed it together.
/// Guarantees: The expanded-path bound applies to the total across all entries.
#[test]
fn bounds_expanded_counter_paths_across_objects() {
    let objects = |count: usize| {
        json!({
            "metrics": {"available": {"description": "Value.", "unit": "By", "gauge": {}}},
            "max_expanded_counters": 256,
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
    let config = Config::from_json(&objects(128)).unwrap();
    assert_eq!(config.counters.len(), 256);
    assert_config_error(objects(129), "exceeding max_expanded_counters 256");
}

fn metric() -> serde_json::Value {
    json!({
        "description": "Test counter.",
        "unit": "By",
        "gauge": {}
    })
}

/// Scenario: Discovery limits or refresh cadence cross their supported boundaries.
/// Guarantees: Invalid options fail pure configuration validation with the relevant field context.
#[test]
fn validates_discovery_option_boundaries() {
    let base = json!({
        "collection_interval": "5s",
        "perfcounters": [{"object": "Memory", "counters": [{"name": "Available Bytes"}]}]
    });
    for (key, value, expected) in [
        (
            "wildcard_refresh_interval",
            json!("0s"),
            "at least collection_interval",
        ),
        (
            "wildcard_refresh_interval",
            json!("4s"),
            "at least collection_interval",
        ),
        (
            "max_instances_per_wildcard",
            json!(0),
            "between 1 and 16384",
        ),
        (
            "max_instances_per_wildcard",
            json!(16385),
            "between 1 and 16384",
        ),
        ("max_instances_per_wildcard", json!(null), "invalid type"),
        ("max_expanded_counters", json!(0), "between 1 and 16384"),
        ("max_expanded_counters", json!(16385), "between 1 and 16384"),
        ("max_expanded_counters", json!(null), "invalid type"),
    ] {
        let mut value_with_option = base.clone();
        value_with_option[key] = value;
        assert_config_error(value_with_option, expected);
    }
    let mut inverted = base;
    inverted["max_instances_per_wildcard"] = json!(3);
    inverted["max_expanded_counters"] = json!(2);
    assert_config_error(
        inverted,
        "max_instances_per_wildcard must not exceed max_expanded_counters",
    );
}

/// Scenario: Explicit discovery options select minimum and maximum valid limits.
/// Guarantees: Normalization preserves configured limits and a refresh cadence equal to collection is accepted.
#[test]
fn accepts_explicit_discovery_limits() {
    for limit in [1, 256, 4096, 16384] {
        let config = Config::from_json(&json!({
            "collection_interval": "5s", "wildcard_refresh_interval": "5s",
            "max_instances_per_wildcard": limit, "max_expanded_counters": limit,
            "perfcounters": [{"object": "Process", "instances": "*", "counters": [{"name": "Private Bytes"}]}]
        })).unwrap();
        assert_eq!(config.max_instances_per_wildcard, limit);
        assert_eq!(config.max_expanded_counters, limit);
        assert_eq!(config.wildcard_refresh_interval(), Duration::from_secs(5));
    }
}

/// Scenario: Objects with no instances, named instances, and wildcards share metric metadata.
/// Guarantees: Public configuration normalizes to exact and wildcard paths without changing metadata.
#[test]
fn normalizes_structured_configuration() {
    let config = Config::from_json(&json!({
        "metrics": {
            "windows.memory.available": metric(),
            "windows.process.time": {
                "description": "Process time.",
                "unit": "%",
                "gauge": {}
            }
        },
        "perfcounters": [
            {
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "windows.memory.available"
                }]
            },
            {
                "object": "Process",
                "instances": ["*", "_Total"],
                "counters": [
                    {
                        "name": "% Processor Time",
                        "metric": "windows.process.time",
                        "attributes": {"state": "active"}
                    },
                    {
                        "name": "% Idle Time",
                        "metric": "windows.process.time",
                        "attributes": {"state": "idle"}
                    }
                ]
            }
        ]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 3);
    assert_eq!(config.counters[0].path, r"\Memory\Available Bytes");
    assert_eq!(config.counters[1].path, r"\Process(*)\% Processor Time");
    assert_eq!(config.counters[1].name.as_ref(), "windows.process.time");
    assert_eq!(config.counters[1].metric_kind, MetricKind::Gauge);
    assert_eq!(config.counters[1].attributes["state"], "active");
    assert_eq!(config.counters[2].attributes["state"], "idle");
    assert!(!config.counters[0].instance_config.has_instance());
    assert!(!config.counters[0].instance_config.emit_attribute());
    assert!(config.counters[1].instance_config.has_instance());
    assert!(config.counters[1].instance_config.emit_attribute());
    assert_eq!(
        config.counters[1].instance_config.excluded_aggregation(),
        None
    );
    assert_eq!(config.collection_interval, Duration::from_secs(60));
    assert_eq!(config.initial_delay, Duration::from_secs(1));
    assert_eq!(config.timeout, None);
    assert_eq!(config.max_instances_per_wildcard, 256);
    assert_eq!(config.max_expanded_counters, 4096);
    assert_eq!(config.wildcard_refresh_interval(), Duration::from_secs(60));
    assert_eq!(config.query_retry_cap(), Duration::from_secs(60));
}

/// Scenario: Timeout is omitted, positive, zero, or negative.
/// Guarantees: Omitted and zero values use the collection interval, a positive timeout takes precedence, and invalid negatives fail.
#[test]
fn normalizes_scrape_timeout() {
    let base = json!({
        "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available Bytes"}]
        }],
        "collection_interval": "45s"
    });

    let omitted = Config::from_json(&base).unwrap();
    assert_eq!(omitted.timeout, None);
    assert_eq!(omitted.scrape_timeout_threshold(), Duration::from_secs(45));

    let mut positive = base.clone();
    positive["timeout"] = json!("10s");
    let positive = Config::from_json(&positive).unwrap();
    assert_eq!(positive.timeout, Some(Duration::from_secs(10)));
    assert_eq!(positive.scrape_timeout_threshold(), Duration::from_secs(10));

    let mut disabled = base.clone();
    disabled["timeout"] = json!("0s");
    let zero = Config::from_json(&disabled).unwrap();
    assert_eq!(zero.timeout, None);
    assert_eq!(zero.scrape_timeout_threshold(), Duration::from_secs(45));

    let mut negative = base;
    negative["timeout"] = json!("-1s");
    assert_config_error(negative, "invalid value");
}

/// Scenario: Wildcard refresh is omitted for short, default, and long collection intervals or set explicitly.
/// Guarantees: Membership scans avoid high-frequency defaults while query recovery remains capped by scrape cadence.
#[test]
fn defaults_refresh_independently_of_query_retry() {
    let config = |collection_interval: &str, refresh_interval: Option<&str>| {
        let mut value = json!({
            "perfcounters": [{
                "object": "Process",
                "instances": ["*"],
                "counters": [{"name": "Private Bytes"}]
            }],
            "collection_interval": collection_interval
        });
        if let Some(refresh_interval) = refresh_interval {
            value["wildcard_refresh_interval"] = json!(refresh_interval);
        }
        Config::from_json(&value)
    };

    let short = config("5s", None).unwrap();
    assert_eq!(short.wildcard_refresh_interval(), Duration::from_secs(60));
    assert_eq!(short.query_retry_cap(), Duration::from_secs(5));

    let default = config("60s", None).unwrap();
    assert_eq!(default.wildcard_refresh_interval(), Duration::from_secs(60));
    assert_eq!(default.query_retry_cap(), Duration::from_secs(60));

    let long = config("2m", None).unwrap();
    assert_eq!(long.wildcard_refresh_interval(), Duration::from_secs(120));
    assert_eq!(long.query_retry_cap(), Duration::from_secs(120));

    assert_eq!(
        config("5s", Some("90s"))
            .unwrap()
            .wildcard_refresh_interval(),
        Duration::from_secs(90)
    );
}

/// Scenario: Mappings explicitly disable or enable query recreation across expanded paths.
/// Guarantees: Explicit false remains supported; true fails with the exact object/counter-indexed diagnostic.
#[test]
fn accepts_disabled_recreation_and_rejects_enabled_recreation() {
    let config = json!({
        "perfcounters": [
            {
                "object": "Memory",
                "instances": ["first", "second"],
                "counters": [
                    {
                        "name": "Available Bytes",
                        "recreate_query": false
                    },
                    {"name": "Committed Bytes"}
                ]
            },
            {"object": "System", "counters": [{"name": "Processes"}, {"name": "Threads"}]}
        ]
    });

    let disabled = Config::from_json(&config).unwrap();
    assert_eq!(disabled.counters.len(), 6);
    assert_eq!(disabled.counters[0].source_mapping_index, 0);
    assert_eq!(disabled.counters[1].source_mapping_index, 0);

    for (object_index, counter_index) in [(0, 0), (1, 1)] {
        let mut invalid = config.clone();
        invalid["perfcounters"][object_index]["counters"][counter_index]["recreate_query"] =
            json!(true);
        assert_eq!(
            Config::from_json(&invalid).unwrap_err().to_string(),
            format!(
                "An invalid user configuration occurred: perfcounters[{object_index}].counters[{counter_index}].recreate_query: true is not supported; query rebuilds after invalidation are automatic"
            )
        );
    }
}

/// Scenario: Implicit exact and wildcard paths match definitions using configured spelling.
/// Guarantees: Metric lookup and native exact paths preserve configured instance indexes.
#[test]
fn resolves_configured_path_spelling_to_metric_definitions() {
    let config = Config::from_json(&json!({
        "metrics": {
            r"\Process(app#001)\Private Bytes": {"unit": "By"},
            r"\Process(worker*)\Working Set": {"unit": "KiBy"}
        },
        "perfcounters": [{
            "object": "Process",
            "instances": ["app#001", "worker*"],
            "counters": [
                {"name": "Private Bytes"},
                {"name": "Working Set"}
            ]
        }]
    }))
    .unwrap();

    assert_eq!(config.counters[0].path, r"\Process(app#001)\Private Bytes");
    assert_eq!(
        config.counters[0].name.as_ref(),
        r"\Process(app#001)\Private Bytes"
    );
    assert_eq!(config.counters[0].unit.as_ref(), "By");
    assert_eq!(
        config.counters[3].name.as_ref(),
        r"\Process(worker*)\Working Set"
    );
    assert_eq!(config.counters[3].unit.as_ref(), "KiBy");
}

/// Scenario: A wildcard omits the default or custom aggregation instance unless selected.
/// Guarantees: Normalization records filtering only when the aggregate was not explicitly included.
#[test]
fn normalizes_aggregation_selection() {
    let excluded = Config::from_json(&json!({
        "metrics": {"test": metric()},
        "perfcounters": [{
            "object": "Custom Object",
            "instances": "*",
            "aggregation_name": "_Global_",
            "counters": [{"name": "Counter", "metric": "test"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        excluded.counters[0].instance_config.excluded_aggregation(),
        Some("_Global_")
    );
    let included = Config::from_json(&json!({
        "metrics": {"test": metric()},
        "perfcounters": [{
            "object": "Custom Object",
            "instances": ["*", "_Global_"],
            "aggregation_name": "_Global_",
            "counters": [{"name": "Counter", "metric": "test"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        included.counters[0].instance_config.excluded_aggregation(),
        None
    );
    assert!(included.counters[0].instance_config.emit_attribute());

    let case_distinct = Config::from_json(&json!({
        "metrics": {"test": metric()},
        "perfcounters": [{
            "object": "Custom Object",
            "instances": ["*", "_GLOBAL_"],
            "aggregation_name": "_Global_",
            "counters": [{"name": "Counter", "metric": "test"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        case_distinct.counters[0]
            .instance_config
            .excluded_aggregation(),
        Some("_Global_")
    );

    let aggregate_alone = Config::from_json(&json!({
        "metrics": {"test": metric()},
        "perfcounters": [{
            "object": "Custom Object",
            "instances": "_Global_",
            "aggregation_name": "_Global_",
            "counters": [{"name": "Counter", "metric": "test"}]
        }]
    }))
    .unwrap();
    assert!(aggregate_alone.counters[0].instance_config.has_instance());
    assert!(!aggregate_alone.counters[0].instance_config.emit_attribute());

    let aggregate_with_peer = Config::from_json(&json!({
        "metrics": {"test": metric()},
        "perfcounters": [{
            "object": "Processor",
            "instances": ["_Total", "worker"],
            "counters": [{"name": "Counter", "metric": "test"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        aggregate_with_peer
            .counters
            .iter()
            .map(|counter| {
                (
                    counter.path.as_str(),
                    counter.instance_config.emit_attribute(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (r"\Processor(_Total)\Counter", false),
            (r"\Processor(worker)\Counter", true)
        ]
    );
}

/// Scenario: Multiple named instances are selected without a wildcard.
/// Guarantees: Each instance becomes one exact path while retaining shared metric identity.
#[test]
fn normalizes_named_instances() {
    let config = Config::from_json(&json!({
        "metrics": {"test": metric()},
        "perfcounters": [{
            "object": "Processor",
            "instances": ["0", "1"],
            "counters": [{"name": "% Processor Time", "metric": "test"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        config
            .counters
            .iter()
            .map(|counter| counter.path.as_str())
            .collect::<Vec<_>>(),
        [
            r"\Processor(0)\% Processor Time",
            r"\Processor(1)\% Processor Time"
        ]
    );
    assert!(
        config
            .counters
            .iter()
            .all(|counter| counter.name.as_ref() == "test")
    );
    assert!(
        config
            .counters
            .iter()
            .all(|counter| counter.instance_config.has_instance())
    );
    assert!(
        config
            .counters
            .iter()
            .all(|counter| counter.instance_config.emit_attribute())
    );
}

/// Scenario: Static `instance` attributes are configured for exact, multiple, wildcard, or ignored mappings.
/// Guarantees: Only effective overrides that can collapse multiple selected instances request a warning.
#[test]
fn identifies_risky_instance_overrides() {
    let override_attributes = BTreeMap::from([("instance".to_owned(), "fixed".to_owned())]);
    let ignored_attributes = BTreeMap::new();

    assert!(!should_warn_instance_override(false, &override_attributes));
    assert!(should_warn_instance_override(true, &override_attributes));
    assert!(should_warn_instance_override(true, &override_attributes));
    assert!(!should_warn_instance_override(true, &ignored_attributes));
}

/// Scenario: Path-shaped definitions are selected implicitly, by a wildcard template, or by an explicit alias.
/// Guarantees: Only raw path-shaped definitions unused by every mapping produce one usage-warning candidate.
#[test]
fn identifies_unused_path_metric_definitions() {
    let value = json!({
        "metrics": {
            r"\Memory\Available Bytes": {"unit": "By"},
            r"\Process(worker*)\Private Bytes": {"unit": "By"},
            r"\alias": {"unit": "By"},
            r"\unused": {"unit": "1"},
            r" \ordinary-leading-space": {"unit": "1"},
            "ordinary": {"unit": "1"}
        },
        "perfcounters": [
            {
                "object": "Memory",
                "counters": [
                    {"name": "Available Bytes"},
                    {"name": "Cache Bytes", "metric": r"\alias"}
                ]
            },
            {
                "object": "Process",
                "instances": "worker*",
                "counters": [{"name": "Private Bytes"}]
            }
        ]
    });
    let user: UserConfig = serde_json::from_value(value.clone()).unwrap();
    let metrics = normalize_metrics(&user.metrics).unwrap();
    let config = Config::from_json(&value).unwrap();
    assert_eq!(
        unused_path_metric_definitions(
            metrics
                .iter()
                .map(|(name, metric)| (name.as_ref(), metric.path_shaped)),
            config.counters.iter().map(|counter| counter.name.as_ref())
        ),
        [r"\unused"]
    );
}

/// Scenario: A valid configuration has a case collision, an unused path definition, and a multi-instance static override.
/// Guarantees: Parsing collects each warning exactly once for emission only after full validation succeeds.
#[test]
fn collects_config_warnings_after_validation() {
    let config = Config::from_json(&json!({
        "metrics": {
            "cpu": {"unit": "1"},
            "CPU": {"unit": "%"},
            r"\Memory\Unused": {"unit": "By"}
        },
        "perfcounters": [{
            "object": "Processor",
            "instances": "*",
            "counters": [{
                "name": "% Processor Time",
                "metric": "cpu",
                "attributes": {"instance": "fixed"}
            }]
        }]
    }))
    .unwrap();

    assert_eq!(config.warnings.len(), 3);
    assert!(matches!(
        &config.warnings[0],
        ConfigWarning::CaseDistinctMetricDefinitions(names)
            if names == &vec!["CPU".to_owned(), "cpu".to_owned()]
    ));
    assert_eq!(
        config.warnings[1],
        ConfigWarning::InstanceOverride {
            object_index: 0,
            counter_index: 0,
        }
    );
    assert_eq!(
        config.warnings[2],
        ConfigWarning::UnusedPathMetricDefinition(r"\Memory\Unused".to_owned())
    );
    config.emit_warnings();
}

/// Scenario: Instances are empty, use `*` or `?` partial wildcards, or combine full wildcard with explicit names.
/// Guarantees: Empty means no instance, partial patterns remain wildcard templates, and only literal `*` selects all instances.
#[test]
fn accepts_go_instance_selection_forms() {
    let empty = Config::from_json(&json!({
        "perfcounters": [{
            "object": "Memory",
            "instances": [],
            "aggregation_name": "",
            "counters": [{"name": "Available Bytes"}]
        }]
    }))
    .unwrap();
    assert_eq!(empty.counters[0].path, r"\Memory\Available Bytes");

    let partial = Config::from_json(&json!({
        "perfcounters": [{
            "object": "Process",
            "instances": ["worker*", "service"],
            "counters": [{"name": "Private Bytes"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        partial
            .counters
            .iter()
            .map(|counter| counter.path.as_str())
            .collect::<Vec<_>>(),
        [
            r"\Process(worker*)\Private Bytes",
            r"\Process(service)\Private Bytes"
        ]
    );
    assert_eq!(
        partial.counters[0].instance_config.excluded_aggregation(),
        Some("_Total")
    );

    let question = Config::from_json(&json!({
        "perfcounters": [{
            "object": "Process",
            "instances": ["svc?ost"],
            "counters": [{"name": "ID Process"}]
        }]
    }))
    .unwrap();
    assert_eq!(question.counters[0].path, r"\Process(svc?ost)\ID Process");
    assert!(question.counters[0].instance_config.is_wildcard());
    assert_eq!(
        question.counters[0].instance_config.excluded_aggregation(),
        Some("_Total")
    );

    let full = Config::from_json(&json!({
        "perfcounters": [{
            "object": "Process",
            "instances": ["*", "worker"],
            "counters": [{"name": "Private Bytes"}]
        }]
    }))
    .unwrap();
    assert_eq!(full.counters.len(), 1);
    assert_eq!(full.counters[0].path, r"\Process(*)\Private Bytes");
}

/// Scenario: Native object, instance, and counter components are empty, or aggregation names are empty or whitespace-only.
/// Guarantees: Required literal empties fail, only an empty aggregation defaults, and whitespace-only spellings remain configured identity.
#[test]
fn handles_empty_native_components_and_aggregation() {
    for value in [
        json!({
            "perfcounters": [{
                "object": "",
                "counters": [{"name": "Available Bytes"}]
            }]
        }),
        json!({
            "perfcounters": [{
                "object": "Process",
                "instances": "",
                "counters": [{"name": "Private Bytes"}]
            }]
        }),
        json!({
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": ""}]
            }]
        }),
    ] {
        assert_config_error(value, "must not be empty");
    }

    let defaulted = Config::from_json(&json!({
        "perfcounters": [{
            "object": "Process",
            "instances": "*",
            "aggregation_name": "",
            "counters": [{"name": "Private Bytes"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        defaulted.counters[0].instance_config.excluded_aggregation(),
        Some("_Total")
    );

    let whitespace = Config::from_json(&json!({
        "perfcounters": [{
            "object": " ",
            "instances": "*",
            "aggregation_name": " ",
            "counters": [{"name": " "}]
        }]
    }))
    .unwrap();
    assert_eq!(whitespace.counters[0].path, r"\ (*)\ ");
    assert_eq!(
        whitespace.counters[0]
            .instance_config
            .excluded_aggregation(),
        Some(" ")
    );
}

/// Scenario: An empty instance list accompanies 257 exact counter mappings and an equally sized native budget.
/// Guarantees: Larger bounded deployments are not rejected by an unrelated legacy template cap.
#[test]
fn allows_exact_paths_beyond_removed_template_cap() {
    let object = |counters: usize| {
        json!({
            "object": "Memory",
            "instances": [],
            "counters": (0..counters)
                .map(|index| json!({"name": format!("Counter {index}")}))
                .collect::<Vec<_>>()
        })
    };
    assert_eq!(
        Config::from_json(&json!({
            "perfcounters": [object(257)]
        }))
        .unwrap()
        .counters
        .len(),
        257
    );
}

/// Scenario: Exact, wildcard and mixed selectors reach or exceed explicit limits, including more than 256 selectors.
/// Guarantees: Every normalized selector counts once; exactly the cap succeeds and the next rejects.
#[test]
fn bounds_all_configured_selectors() {
    for cap in [1, 16, 257] {
        for kind in ["exact", "wildcard", "mixed"] {
            for excess in [0, 1] {
                let instances = (0..cap + excess)
                    .map(|index| {
                        if kind == "wildcard" || (kind == "mixed" && index % 2 == 0) {
                            format!("worker{index}*")
                        } else {
                            format!("worker{index}")
                        }
                    })
                    .collect::<Vec<_>>();
                let mut value = json!({
                    "max_instances_per_wildcard": 1,
                    "perfcounters": [{
                        "object": "Process",
                        "instances": instances,
                        "counters": [{"name": "ID Process"}]
                    }]
                });
                value["max_expanded_counters"] = json!(cap);
                if excess == 0 {
                    let config = Config::from_json(&value).unwrap();
                    assert_eq!(config.counters.len(), cap);
                    for (index, counter) in config.counters.iter().enumerate() {
                        assert_eq!(
                            counter.instance_config.is_wildcard(),
                            kind == "wildcard" || (kind == "mixed" && index % 2 == 0)
                        );
                    }
                } else {
                    assert_eq!(
                        Config::from_json(&value).unwrap_err().to_string(),
                        format!(
                            "An invalid user configuration occurred: perfcounters normalize to {} counter selectors, exceeding max_expanded_counters {cap}",
                            cap + 1
                        ),
                    );
                }
            }
        }
    }
}

/// Scenario: Many wildcard patterns and counter mappings would generate 65,536 selectors.
/// Guarantees: Normalization rejects at the default 4097th selector instead of materializing the product.
#[test]
fn rejects_large_wildcard_selector_product() {
    let error = Config::from_json(&json!({"perfcounters": [{
        "object": "Process",
        "instances": (0..256).map(|i| format!("worker{i}*")).collect::<Vec<_>>(),
        "counters": (0..256).map(|i| json!({"name": format!("Counter {i}")})).collect::<Vec<_>>()
    }]}))
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "An invalid user configuration occurred: perfcounters normalize to 4097 counter selectors, exceeding max_expanded_counters 4096",
    );
}

/// Scenario: One definition is expanded, referenced again, and the normalized config is cloned.
/// Guarantees: Name, unit, description, and per-mapping attributes retain their shared backing.
#[test]
fn shares_expanded_counter_metadata() {
    let config = Config::from_json(&json!({
        "metrics": {"test": metric()},
        "perfcounters": [
            {
                "object": "Process",
                "instances": ["a", "b"],
                "counters": [{
                    "name": "Private Bytes",
                    "metric": "test",
                    "attributes": {"state": "used"}
                }]
            },
            {
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "test"}]
            }
        ]
    }))
    .unwrap();
    let [first, second, repeated] = config.counters.as_slice() else {
        panic!("expected two expanded counters and one repeated reference");
    };
    assert!(Arc::ptr_eq(&first.name, &second.name));
    assert!(Arc::ptr_eq(&first.unit, &second.unit));
    assert!(Arc::ptr_eq(&first.description, &second.description));
    assert!(Arc::ptr_eq(&first.attributes, &second.attributes));
    assert!(Arc::ptr_eq(&first.name, &repeated.name));
    assert!(Arc::ptr_eq(&first.unit, &repeated.unit));
    assert!(Arc::ptr_eq(&first.description, &repeated.description));

    let cloned = config.clone();
    assert!(Arc::ptr_eq(&first.name, &cloned.counters[0].name));
    assert!(Arc::ptr_eq(&first.unit, &cloned.counters[0].unit));
    assert!(Arc::ptr_eq(
        &first.description,
        &cloned.counters[0].description
    ));
    assert!(Arc::ptr_eq(
        &first.attributes,
        &cloned.counters[0].attributes
    ));
}

/// Scenario: Invalid scales belong to fixed-instance or wildcard mappings that also have an undefined metric.
/// Guarantees: Scale validation runs before metadata lookup and reports the first configured path without changing its spelling.
#[test]
fn invalid_scale_identifies_the_first_configured_path() {
    for (instances, path) in [
        (json!(["worker ", "other"]), r"\Process(worker )\ID Process"),
        (json!(["*"]), r"\Process(*)\ID Process"),
        (json!(["svchos?"]), r"\Process(svchos?)\ID Process"),
    ] {
        let error = Config::from_json(&json!({"perfcounters": [{
            "object": "Process",
            "instances": instances,
            "counters": [{"name": "ID Process", "scale_power10": 19, "metric": "missing"}]
        }]}))
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "An invalid user configuration occurred: counter path {path:?} scale_power10 must be between -18 and 18"
            )
        );
    }
}
