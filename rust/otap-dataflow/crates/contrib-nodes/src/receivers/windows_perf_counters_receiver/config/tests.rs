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

/// Scenario: A referenced metric omits its description or supplies empty or whitespace text.
/// Guarantees: An omitted description defaults to empty; supplied descriptions remain unchanged.
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
        assert_eq!(
            config.counters[0].description.as_ref(),
            description.unwrap_or("")
        );
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
/// Guarantees: Only an empty unit receives the "1" default; whitespace-only units remain unchanged.
#[test]
fn defaults_empty_metric_unit() {
    for unit in ["", " \t "] {
        let mut value = config_with_timing("30s", "1s");
        value["metrics"]["available"]["unit"] = json!(unit);
        let config = RuntimeConfig::from_json(&value).unwrap();
        assert_eq!(
            config.counters[0].unit,
            if unit.is_empty() { "1" } else { unit }
        );
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
        let config = RuntimeConfig::from_json(&json!({
            "metrics": {name: {"unit": "By"}},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": name}]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters[0].name, name);
        assert_eq!(config.counters[0].unit, "By");
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
        let config = RuntimeConfig::from_json(&unit_config(unit)).unwrap();
        assert_eq!(config.counters[0].unit, unit);
    }
}

/// Scenario: A metric name, its exact reference and its unit contain control characters.
/// Guarantees: Metric names, references and units retain their configured control characters.
/// Metric metadata is not a native PDH selector and must not inherit native-component restrictions.
#[test]
fn preserves_control_characters_in_metric_names_references_and_units() {
    let name = "metric\nname";
    let unit = "B\0y";
    let config = RuntimeConfig::from_json(&json!({
        "metrics": {name: {"unit": unit}},
        "perfcounters": [{"object": "Memory", "counters": [
            {"name": "Available Bytes", "metric": name}
        ]}]
    }))
    .unwrap();
    assert_eq!(config.counters[0].name, name);
    assert_eq!(config.counters[0].unit, unit);
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
    let config = RuntimeConfig::from_json(&value).unwrap();
    assert_eq!(
        config.counters[0].description.as_ref(),
        "Available memory.\n"
    );
}

/// Scenario: Definitions exceed the configured native-path budget but only one is referenced.
/// Guarantees: Unused definitions do not consume the counter-path budget or produce counters.
#[test]
fn accepts_metric_definitions_beyond_path_limit() {
    let metrics = (0..=MAX_COUNTERS)
        .map(|index| {
            (
                format!("m{index}"),
                json!({"description": "Value.", "unit": "By", "gauge": {}}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let config = RuntimeConfig::from_json(&json!({
        "metrics": metrics,
        "perfcounters": [{
            "object": "Memory",
            "counters": [{"name": "Available Bytes", "metric": "m0"}]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 1);
    assert_eq!(config.counters[0].name, "m0");
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
            let config = RuntimeConfig::from_json(&value).unwrap();
            for (counter, expected_path) in [
                (&config.counters[0], r"\Process(app#00)\Private Bytes"),
                (&config.counters[1], r"\Process(worker#001)\Private Bytes"),
            ] {
                assert_eq!(counter.name, expected_path);
                assert_eq!(counter.path, expected_path);
                assert_eq!(counter.kind, MetricKind::Gauge);
                assert_eq!(counter.unit, "1");
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
        let config = RuntimeConfig::from_json(&value).unwrap();
        assert_eq!(config.counters[0].name, path);
        assert_eq!(
            config.counters[0].unit,
            if name == path { "exact" } else { "1" }
        );
        value["perfcounters"][0]["counters"][0]["metric"] = json!(name);
        let config = RuntimeConfig::from_json(&value).unwrap();
        assert_eq!(config.counters[0].name, name);
        assert_eq!(config.counters[0].unit, "exact");
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
    let config = RuntimeConfig::from_json(&value).unwrap();
    assert!(
        config
            .counters
            .iter()
            .all(|counter| counter.unit == "selected")
    );
    assert_eq!(
        unused_path_metric_definitions(
            value["metrics"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str),
            &config.counters,
        ),
        [overlong.as_str(), r"\unavailable\counter",]
    );
    value["perfcounters"][0]["counters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "Committed Bytes", "metric": overlong}));
    let config = RuntimeConfig::from_json(&value).unwrap();
    assert_eq!(
        unused_path_metric_definitions(
            value["metrics"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str),
            &config.counters,
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
        case_distinct_metric_definitions(names),
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
    let config = RuntimeConfig::from_json(&value).unwrap();
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
        assert_eq!(counter.name, name);
        assert_eq!(counter.unit, unit);
        assert_eq!(counter.kind, kind);
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
    let config = RuntimeConfig::from_json(&json!({
        "metrics": null,
        "perfcounters": [{"object": "Memory", "counters": [{"name": "Available Bytes"}]}]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 1);
    let counter = &config.counters[0];
    assert_eq!(counter.name, r"\Memory\Available Bytes");
    assert_eq!(counter.unit, "1");
    assert!(counter.description.is_empty());
    assert_eq!(counter.kind, MetricKind::Gauge);
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

/// Scenario: An object, instance or counter component is a literal wildcard in this exact-only receiver.
/// Guarantees: Preserving spelling never admits object patterns, instance discovery or counter expansion.
#[test]
fn rejects_wildcard_components() {
    for pattern in ["*", "?"] {
        for (object, instance, counter, field) in [
            (pattern, "app", "Value", "perfcounters[0].object"),
            ("Process", pattern, "Value", "perfcounters[0].instances"),
            (
                "Process",
                "app",
                pattern,
                "perfcounters[0].counters[0].name",
            ),
        ] {
            assert_config_error(
                json!({"perfcounters": [{
                    "object": object, "instances": instance, "counters": [{"name": counter}]
                }]}),
                &format!("{field} contains a reserved"),
            );
        }
    }
}

/// Scenario: Instance indexes use omitted, zero, or leading-zero spellings.
/// Guarantees: Each spelling is accepted individually and retained in native paths and exact implicit lookup.
#[test]
fn preserves_instance_indexes_in_paths_and_lookup() {
    for (instance, unit) in [
        ("app", "By"),
        ("app#0", "KiBy"),
        ("app#00", "MiBy"),
        ("app#1", "GiBy"),
        ("app#01", "TiBy"),
        ("app#001", "PiBy"),
    ] {
        let config = RuntimeConfig::from_json(&json!({
            "metrics": {
                r"\Process(app)\Private Bytes": {"unit": "By"},
                r"\Process(app#0)\Private Bytes": {"unit": "KiBy"},
                r"\Process(app#00)\Private Bytes": {"unit": "MiBy"},
                r"\Process(app#1)\Private Bytes": {"unit": "GiBy"},
                r"\Process(app#01)\Private Bytes": {"unit": "TiBy"},
                r"\Process(app#001)\Private Bytes": {"unit": "PiBy"}
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
        assert_eq!(counter.name, counter.path);
        assert_eq!(counter.unit, unit);
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
    let config = RuntimeConfig::from_json(&json!({"perfcounters": [{
        "object": "Process",
        "instances": instances,
        "counters": [{"name": "Private Bytes"}]
    }]}))
    .unwrap();
    assert_eq!(config.counters.len(), instances.len());
    for (counter, instance) in config.counters.iter().zip(instances) {
        assert_eq!(counter.path, format!(r"\Process({instance})\Private Bytes"));
        assert_eq!(counter.name, counter.path);
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
/// Guarantees: Padding is retained in paths and remains distinct during duplicate detection.
#[test]
fn preserves_surrounding_whitespace_in_path_segments() {
    let config = RuntimeConfig::from_json(&gauge_config(json!({
        "object": " Process ",
        "instances": [" app "],
        "counters": [{"name": " Private Bytes ", "metric": "available"}]
    })))
    .unwrap();
    assert_eq!(
        config.counters[0].path,
        r"\ Process ( app )\ Private Bytes "
    );

    let config = RuntimeConfig::from_json(&gauge_config(json!({
        "object": "Process",
        "instances": ["app", "app "],
        "counters": [{"name": "Private Bytes", "metric": "available"}]
    })))
    .unwrap();
    assert_eq!(config.counters[0].path, r"\Process(app)\Private Bytes");
    assert_eq!(config.counters[1].path, r"\Process(app )\Private Bytes");
    let config = RuntimeConfig::from_json(&json!({
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
        let config = RuntimeConfig::from_json(&json!({"perfcounters": [{
            "object": object, "instances": instance, "counters": [{"name": counter}]
        }]}))
        .unwrap();
        let path = format!("\\{object}({instance})\\{counter}");
        assert_eq!(config.counters[0].path, path);
        assert_eq!(config.counters[0].name, path);
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
    let _config = RuntimeConfig::from_json(&gauge_config(json!({
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

/// Scenario: Metric names, references, descriptions, units, and attribute keys are padded.
/// Guarantees: Metric metadata, attribute keys, and string values reach CounterConfig unchanged.
/// The receiver must not rewrite user-supplied metric metadata.
#[test]
fn preserves_metric_identities_and_metadata() {
    let config = RuntimeConfig::from_json(&json!({
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
    assert_eq!(counter.name, " available ");
    assert_eq!(&*counter.description, " Available\tmemory. ");
    assert_eq!(counter.unit, "\r By \t");
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
        let config = RuntimeConfig::from_json(&json!({
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
            assert_eq!(counter.name, name);
            assert_eq!(counter.unit, unit);
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
        let config = RuntimeConfig::from_json(&value).unwrap();
        assert_eq!(config.counters[0].name, name);
        assert_eq!(config.counters[0].unit, unit);
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
