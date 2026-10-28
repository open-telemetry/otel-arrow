// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::config::{Config, EndpointConfig};
use super::error::MappingError;
use super::event_mapping::{DropReason, MappedLog, MappingContext, MappingOutcome};
use super::field_mapping::{CompiledEndpoint, load_endpoints};
use otel_arrow_dfe_config::data_limits::{
    Budget, CONFIG_FILE_BYTES, DataLimitError, Limits, inspect_json, json_size, read_config,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn no_environment(_: &str) -> Result<String, std::env::VarError> {
    Err(std::env::VarError::NotPresent)
}

/// Scenario: wildcard parents consume literal dotted keys before virtual child-key validation.
/// Guarantees: dotted wildcard boundaries cannot bypass allowed children or reject explicitly allowed real children.
#[test]
fn wildcard_parent_boundaries_enforce_allowed_keys() {
    let ep = endpoint("rules: [{ path: 'groups.*.parent', allowed_keys: [ok] }]");
    for event in [
        json!({"groups": {"foo.bar": {"parent.bad": 1}}}),
        json!({"groups.foo.bar": {"parent.bad": 1}}),
        json!({"groups": {"foo.bar": {"parent": {"bad": 1}}}}),
    ] {
        let result = ep.map_event(&event, &MappingContext::default());
        assert!(
            matches!(result, Err(MappingError::Rejected(_))),
            "{event}: {result:?}"
        );
    }
    let event = json!({"groups": {"foo.bar": {"parent.ok": 1}}});
    assert_eq!(accepted(&ep, event.clone()).body, Some(event));
    let ep = endpoint("rules: [{ path: 'groups.*', allowed_keys: [safe] }]");
    for event in [
        json!({"groups": {"foo.bar": {"safe": 1}}}),
        json!({"groups.foo.bar": {"safe": 1}}),
    ] {
        assert_eq!(accepted(&ep, event.clone()).body, Some(event));
    }
}

/// Scenario: many additional drop targets receive one large validated string.
/// Guarantees: untransformed fanout borrows its input and does not copy payload-sized values for each target.
#[test]
fn untransformed_drop_fanout_borrows_large_values() {
    let targets: Vec<_> = (0..4096)
        .map(|_| json!({"target": {"kind": "drop"}}))
        .collect();
    let config = serde_json::from_value(json!({"rules": [{"path": "large", "also": targets}]}))
        .expect("schema");
    let ep = CompiledEndpoint::compile(&config, None, &no_environment).expect("compile");
    assert!(
        ep.map_event(
            &json!({"large": "x".repeat(2 * 1024 * 1024)}),
            &MappingContext::default()
        )
        .is_ok()
    );
}

/// Scenario: a drop container and explicitly mapped child are encoded with dotted keys instead of real parents.
/// Guarantees: virtual dropped ancestors suppress hidden body/retained-source fields while explicit children still map.
#[test]
fn virtual_drop_parents_suppress_unmapped_descendants() {
    let ep = endpoint(
        "filtered_item_size: true\nrules:\n  - path: secret\n    target: { kind: drop }\n  - path: secret.visible\n    target: { kind: attribute, key: visible }",
    );
    let event = json!({"secret.hidden": "private", "secret.visible": true});
    let log = accepted(&ep, event);
    assert_eq!(log.attributes["visible"], true);
    assert!(log.body.is_none());
    assert_eq!(
        log.attributes["filtered_item_size"],
        json!(
            serde_json::to_vec(&json!({"secret.visible": true}))
                .expect("serialize")
                .len()
        )
    );
    let ep = endpoint(
        "rules: [{ path: secret, validate: {} }, { path: secret, target: { kind: drop } }]",
    );
    assert_eq!(
        accepted(&ep, json!({"secret.hidden": "kept"})).body,
        Some(json!({"secret.hidden": "kept"}))
    );
}

/// Scenario: inherited rules contain large owned target strings in each supported string-bearing target option.
/// Guarantees: compilation refuses expansion before cloning those bytes beyond the startup work allowance.
#[test]
fn inherited_target_strings_are_fully_budgeted() {
    let large = "x".repeat(512 * 1024);
    for target in [
        json!({"kind": "event_name", "split_last": large}),
        json!({"kind": "event_name", "strip_segment": {"index": 1, "match_field": large}}),
        json!({"kind": "severity", "map": {large.clone(): {"number": 1, "text": "TRACE"}}}),
        json!({"kind": "scope", "field": "name_version", "split": {"last_char": large}}),
    ] {
        let config: EndpointConfig =
            serde_json::from_value(json!({"rules": [{"path": "value", "target": target}]}))
                .expect("schema");
        assert!(CompiledEndpoint::compile(&config, None, &no_environment).is_ok());
        let mut expanded = config;
        expanded.event_overrides =
            serde_json::from_value(json!({"A": {}, "B": {}, "C": {}, "D": {}})).expect("overrides");
        assert!(matches!(
            CompiledEndpoint::with_limits(
                &expanded,
                None,
                &no_environment,
                Limits {
                    work: 2 * 1024 * 1024,
                    ..Limits::default()
                }
            ),
            Err(MappingError::Limit("traversal work"))
        ));
    }
}

/// Scenario: the selected event-name segment appears first, in the middle, last or as the entire name.
/// Guarantees: matching final segments remove their preceding separator; nonmatching segments remain unchanged.
#[test]
fn event_name_stripping_handles_final_segments() {
    for (index, input, key, expected) in [
        (0, "key.Event", "key", "Event"),
        (1, "Event.key.Tail", "key", "Event.Tail"),
        (1, "Event.key", "key", "Event"),
        (0, "key", "key", ""),
        (1, "Event.key", "different", "Event.key"),
        (4, "Event.key", "key", "Event.key"),
    ] {
        let ep = endpoint(&format!(
            "rules: [{{ path: name, target: {{ kind: event_name, strip_segment: {{ index: {index}, match_field: iKey }} }} }}]"
        ));
        assert_eq!(
            accepted(&ep, json!({"name": input, "iKey": key}))
                .event_name
                .as_deref(),
            Some(expected)
        );
    }
}

/// Scenario: allowed children include a literal dotted name and events use actual or virtual containers.
/// Guarantees: explicitly listed literal keys survive descendant traversal, while unlisted and genuinely nested children are refused.
#[test]
fn allowed_keys_preserve_literal_child_boundaries() {
    let ep = endpoint("rules: [{ path: parent, allowed_keys: [foo.bar] }]");
    for event in [
        json!({"parent": {"foo.bar": 1}}),
        json!({"parent": {"foo.bar": {"nested": 1}}}),
        json!({"parent.foo.bar": 1}),
        json!({"parent.foo.bar": {"nested": 1}}),
    ] {
        let log = accepted(&ep, event.clone());
        assert_eq!(log.body, Some(event));
    }
    for event in [
        json!({"parent": {"foo": {"bar": 1}}}),
        json!({"parent": {"foo.baz": 1}}),
        json!({"parent.foo.baz": 1}),
        json!({"parent.foo": {"bar": 1}}),
    ] {
        assert!(
            matches!(
                ep.map_event(&event, &MappingContext::default()),
                Err(MappingError::Rejected(_))
            ),
            "{event}"
        );
    }
    let ep = endpoint("rules: [{ path: parent, allowed_keys: [foo] }]");
    for event in [
        json!({"parent": {"foo": {"bar": 1}}}),
        json!({"parent": {"foo.bar": 1}}),
        json!({"parent.foo.bar": 1}),
        json!({"parent.foo": {"bar": 1}}),
    ] {
        assert_eq!(accepted(&ep, event.clone()).body, Some(event));
    }
    let ep = endpoint("rules: [{ path: 'outer.parent', allowed_keys: [foo.bar] }]");
    for event in [
        json!({"outer": {"parent": {"foo.bar": 1}}}),
        json!({"outer.parent.foo.bar": 1}),
        json!({"outer": {"parent.foo.bar": 1}}),
    ] {
        assert_eq!(accepted(&ep, event.clone()).body, Some(event));
    }
}

/// Scenario: a long child key is checked against a large allow-list, with and without an exact match.
/// Guarantees: the prefix is scanned once, comparison work is linear in allow-list bytes, and exact budget boundaries are enforced.
#[test]
fn allowed_key_scans_have_linear_budgeted_work() {
    use super::event_mapping::allowed_child;
    let key = "x".repeat(1024);
    let mut keys: Vec<String> = (0..20_000).map(|i| format!("k{i}")).collect();
    let work = key.len() + 1 + keys.iter().map(|s| 2 * s.len() + 1).sum::<usize>();
    let mut budget = Budget::new(work);
    assert_eq!(allowed_child(&keys, &key, &mut budget), Ok(false));
    assert_eq!(budget.remaining(), 0);
    assert_eq!(
        allowed_child(&keys, &key, &mut Budget::new(work - 1)),
        Err(MappingError::Limit("traversal work"))
    );
    keys.push(key.clone());
    let work = work + 2 * key.len() + 1;
    assert_eq!(allowed_child(&keys, &key, &mut Budget::new(work)), Ok(true));
    let config =
        serde_json::from_value(json!({"rules": [{"path": "parent", "allowed_keys": keys}]}))
            .expect("schema");
    let ep = CompiledEndpoint::compile(&config, None, &no_environment).expect("compile");
    let event = json!({"parent": {key: "kept"}});
    assert_eq!(accepted(&ep, event.clone()).body, Some(event));
}

/// Scenario: trace and span identifiers contain only zeroes or combine an absent ID with a valid ID.
/// Guarantees: zero IDs do not reject the event, populate context or erase another valid mapping; log content survives.
#[test]
fn zero_context_ids_are_treated_as_not_provided() {
    let ep = endpoint(
        "rules: [{ path: trace, target: { kind: trace_id } }, { path: span, target: { kind: span_id } }]",
    );
    for (trace, span, expected_trace, expected_span) in [
        ("0".repeat(32), "0".repeat(16), None, None),
        ("0".repeat(32), "01".repeat(8), None, Some([1; 8])),
        ("01".repeat(16), "0".repeat(16), Some([1; 16]), None),
        (
            "00".repeat(15) + "0a",
            "00".repeat(7) + "0B",
            Some({
                let mut id = [0; 16];
                id[15] = 10;
                id
            }),
            Some({
                let mut id = [0; 8];
                id[7] = 11;
                id
            }),
        ),
    ] {
        let log = accepted(
            &ep,
            json!({"trace": trace, "span": span, "message": "kept"}),
        );
        assert_eq!(log.trace_id, expected_trace);
        assert_eq!(log.span_id, expected_span);
        assert_eq!(log.body, Some(json!({"message": "kept"})));
    }
    let ep = endpoint(
        "rules: [{ path: a, target: { kind: trace_id } }, { path: b, target: { kind: trace_id } }, { path: c, target: { kind: span_id } }, { path: d, target: { kind: span_id } }]",
    );
    let log = accepted(
        &ep,
        json!({"a": "01".repeat(16), "b": "0".repeat(32), "c": "01".repeat(8), "d": "0".repeat(16)}),
    );
    assert_eq!(log.trace_id, Some([1; 16]));
    assert_eq!(log.span_id, Some([1; 8]));
}

/// Scenario: deep wildcard rules are checked against deep source paths with a limited traversal budget.
/// Guarantees: matching charges its multiplicative DP cost at both primary and allowed-key call sites, without timing assertions.
#[test]
fn wildcard_matching_charges_multiplicative_work() {
    let path = std::iter::repeat_n("x", 32).collect::<Vec<_>>().join(".");
    let config: EndpointConfig = serde_json::from_value(json!({
        "rules": [{"path": path, "target": {"kind": "drop"}}]
    }))
    .expect("schema");
    let ep = CompiledEndpoint::compile(&config, None, &no_environment).expect("compile");
    let rule = &ep.rules[0];
    let source = vec!["x".to_owned(); 32];
    let mut budget = Budget::new(1_000_000);
    assert_eq!(rule.matches(&source, &mut budget), Ok(true));
    let charged = 1_000_000 - budget.remaining();
    assert!(charged >= 32 * 32);
    assert_eq!(
        rule.matches(&source, &mut Budget::new(charged - 1)),
        Err(MappingError::Limit("traversal work"))
    );
    assert_eq!(rule.matches(&source, &mut Budget::new(charged)), Ok(true));
    let mut dotted = Budget::new(charged);
    assert_eq!(rule.matches(&[path], &mut dotted), Ok(true));

    let deep = std::iter::repeat_n("*", 31).collect::<Vec<_>>().join(".");
    for metadata_only in [false, true] {
        let rules: Vec<_> = (0..8)
            .map(|i| {
                if metadata_only {
                    json!({"path": format!("r{i}.{deep}"), "allowed_keys": ["x"]})
                } else {
                    json!({"path": format!("r{i}.{deep}"), "target": {"kind": "drop"}})
                }
            })
            .collect();
        let config = serde_json::from_value(json!({"rules": rules})).expect("schema");
        let mut ep = CompiledEndpoint::compile(&config, None, &no_environment).expect("compile");
        let mut event = json!({"message": "kept"});
        for _ in 0..33 {
            event = json!({"x": event});
        }
        assert!(ep.map_event(&event, &MappingContext::default()).is_ok());
        ep.limits.work = 100_000;
        assert_eq!(
            ep.map_event(&event, &MappingContext::default()),
            Err(MappingError::Limit("traversal work"))
        );
    }
}

/// Scenario: inherited override rules compare long common-prefix paths under a constrained compiler budget.
/// Guarantees: startup merging accounts for path comparison depth instead of charging only candidate count.
#[test]
fn override_merging_charges_path_comparisons() {
    let prefix = std::iter::repeat_n("segment", 15)
        .collect::<Vec<_>>()
        .join(".");
    let rules: Vec<_> = (0..16)
        .map(|i| json!({"path": format!("{prefix}.field{i}"), "target": {"kind": "drop"}}))
        .collect();
    let mut config: EndpointConfig =
        serde_json::from_value(json!({"rules": rules})).expect("schema");
    let limits = Limits {
        work: 40_000,
        ..Limits::default()
    };
    assert!(CompiledEndpoint::with_limits(&config, None, &no_environment, limits).is_ok());
    config.event_overrides =
        serde_json::from_value(json!({"Evt": {"rules": rules}})).expect("overrides");
    assert!(CompiledEndpoint::compile(&config, None, &no_environment).is_ok());
    assert!(matches!(
        CompiledEndpoint::with_limits(&config, None, &no_environment, limits),
        Err(MappingError::Limit("traversal work"))
    ));
}

/// Scenario: accepted media types contain malformed tokens or valid mixed-case parameterized types.
/// Guarantees: invalid type/subtype syntax fails compilation while valid types normalize and ignore parameters.
#[test]
fn accepted_media_types_require_valid_tokens() {
    for media_type in [
        "application/json garbage",
        "application /json",
        "application/ json",
        "/json",
        "application/",
        "application/json/extra",
        "application/(json)",
        "application/j\u{00e9}son",
        "application/json\r\nextra",
        ";charset=utf-8",
    ] {
        let config = serde_json::from_value(json!({"accepted_content_types": [media_type]}))
            .expect("schema");
        assert!(
            matches!(
                CompiledEndpoint::compile(&config, None, &no_environment),
                Err(MappingError::Config(_))
            ),
            "{media_type}"
        );
    }
    let ep = endpoint(
        "accepted_content_types: [' Application/JSON; charset=utf-8 ', 'Application/vnd.example+json; profile=\"logs\"', 'application/x-ndjson']",
    );
    assert_eq!(
        ep.config.accepted_content_types,
        [
            "application/json",
            "application/vnd.example+json",
            "application/x-ndjson"
        ]
    );
}

/// Scenario: wildcard trace rules receive nested, partially dotted and fully dotted SDK fields.
/// Guarantees: equivalent encodings populate identical trace context rather than silently retaining it in the body.
#[test]
fn wildcard_trace_paths_support_dotted_keys() {
    let ep = endpoint("rules: [{ path: 'ext.*.traceId', target: { kind: trace_id } }]");
    let trace = "01010101010101010101010101010101";
    for event in [
        json!({"ext": {"dt": {"traceId": trace}}}),
        json!({"ext": {"dt.traceId": trace}}),
        json!({"ext.dt": {"traceId": trace}}),
        json!({"ext.dt.traceId": trace}),
    ] {
        assert_eq!(accepted(&ep, event).trace_id, Some([1; 16]));
    }
    for event in [
        json!({"ext.dt.notTraceId": trace}),
        json!({"ext.dt.traceId.extra": trace}),
    ] {
        assert!(accepted(&ep, event).trace_id.is_none());
    }
}

/// Scenario: implicit wildcard destination keys receive equivalent dotted and nested source paths.
/// Guarantees: suffixes are independent of JSON key boundaries and literal dotted wildcard keys remain intact.
#[test]
fn wildcard_attribute_suffixes_support_dotted_keys() {
    for target in ["attribute", "resource"] {
        let ep = endpoint(&format!(
            "rules: [{{ path: 'tags.foo.*', target: {{ kind: {target}, prefix: 'p.' }} }}]"
        ));
        for event in [
            json!({"tags": {"foo": {"bar": "value"}}}),
            json!({"tags": {"foo.bar": "value"}}),
            json!({"tags.foo": {"bar": "value"}}),
            json!({"tags.foo.bar": "value"}),
        ] {
            let log = accepted(&ep, event);
            let attrs = if target == "attribute" {
                &log.attributes
            } else {
                &log.resource
            };
            assert_eq!(attrs.get("p.bar"), Some(&json!("value")));
            assert_eq!(attrs.len(), 1);
        }
    }
    let ep = endpoint("rules: [{ path: 'tags.*', target: { kind: attribute } }]");
    for event in [
        json!({"tags": {"ai.user.id": "value"}}),
        json!({"tags.ai.user.id": "value"}),
    ] {
        assert_eq!(accepted(&ep, event).attributes["ai.user.id"], "value");
    }
    let ep = endpoint("rules: [{ path: 'tags.*.value', target: { kind: attribute } }]");
    let log = accepted(&ep, json!({"tags": {"foo.bar": {"value": "kept"}}}));
    assert_eq!(log.attributes["foo.bar.value"], "kept");
}

/// Scenario: a targetless leaf has no metadata, while legitimate validation and inherited overlays omit targets.
/// Guarantees: meaningless leaf rules fail compilation without breaking explicit drops, container traversal or metadata overlays.
#[test]
fn targetless_leaves_require_meaningful_metadata() {
    for yaml in [
        "rules: [{ path: secret }]",
        "rules: [{ path: secret, children: [] }]",
        "rules: [{ path: parent, children: [{ path: secret }] }]",
        "event_overrides: { E: { rules: [{ path: secret }] } }",
    ] {
        let config = serde_yaml::from_str(yaml).expect("schema");
        assert!(matches!(
            CompiledEndpoint::compile(&config, None, &no_environment),
            Err(MappingError::Config(_))
        ));
    }
    let ep = endpoint("rules: [{ path: secret, target: { kind: drop } }]");
    assert!(accepted(&ep, json!({"secret": "value"})).body.is_none());
    let ep = endpoint("rules: [{ path: value, validate: { expected_type: string } }]");
    assert_eq!(
        accepted(&ep, json!({"value": "kept"})).body,
        Some(json!({"value": "kept"}))
    );
    let ep = endpoint(
        "rules:\n  - path: name\n    target: { kind: event_name }\n  - path: value\n    target: { kind: attribute, key: kept }\nevent_overrides:\n  E:\n    rules:\n      - path: value\n        validate: { expected_type: string }",
    );
    assert_eq!(
        accepted(&ep, json!({"name": "E", "value": "kept"})).attributes["kept"],
        "kept"
    );
}

/// Scenario: multiple fields, wildcards or additional targets can produce competing event names.
/// Guarantees: compilation refuses ambiguous names before overrides or iKey allow-lists can check a different name.
#[test]
fn ambiguous_event_name_targets_fail_compilation() {
    for policy in [
        "event_overrides: { Blocked: { action: reject } }",
        "ikey_filter: { keys: { key: { action: accept, allowed_events: [Allowed] } } }",
    ] {
        let yaml = format!(
            r#"
rules:
  - path: first
    target: {{ kind: event_name }}
  - path: second
    target: {{ kind: event_name }}
{policy}
"#,
        );
        let config = serde_yaml::from_str(&yaml).expect("schema");
        assert!(matches!(
            CompiledEndpoint::compile(&config, None, &no_environment),
            Err(MappingError::Config(_))
        ));
    }
    for yaml in [
        "rules: [{ path: 'names.*', target: { kind: event_name } }]",
        "rules: [{ path: name, also: [{ target: { kind: event_name } }] }]",
        "rules: [{ path: name, target: { kind: event_name } }]\nevent_overrides: { Evt: { rules: [{ path: other, target: { kind: event_name } }] } }",
        "event_overrides: { Evt: { inherit_rules: false, rules: [{ path: a, target: { kind: event_name } }, { path: b, target: { kind: event_name } }] } }",
    ] {
        let config = serde_yaml::from_str(yaml).expect("schema");
        assert!(
            matches!(
                CompiledEndpoint::compile(&config, None, &no_environment),
                Err(MappingError::Config(_))
            ),
            "{yaml}"
        );
    }
}

/// Scenario: an override changes the event name after admission policies have selected its original name.
/// Guarantees: name-changing overrides reject the event rather than bypass policies; unchanged names remain valid.
#[test]
fn policy_selected_names_must_match_emitted_names() {
    for policy in [
        "",
        "ikey_filter: { keys: { key: { action: accept, allowed_events: [Allowed] } } }",
    ] {
        let ep = endpoint(&format!(
            "rules: [{{ path: name, target: {{ kind: event_name }} }}]\nevent_overrides:\n  Allowed:\n    rules: [{{ path: name, target: {{ kind: event_name, prefix: 'Blocked.' }} }}]\n{policy}"
        ));
        assert!(matches!(
            ep.map_event(
                &json!({
                    "name": "Allowed", "iKey": "key"
                }),
                &MappingContext::default()
            ),
            Err(MappingError::Rejected(_))
        ));
        let ep = endpoint(&format!(
            "rules: [{{ path: name, target: {{ kind: event_name }} }}]\nevent_overrides: {{ Blocked: {{ action: reject }} }}\n{policy}"
        ));
        let log = accepted(&ep, json!({"name": "Allowed", "iKey": "key"}));
        assert_eq!(log.event_name.as_deref(), Some("Allowed"));
    }
}

/// Scenario: no event limit is configured and a source field exceeds the independent output budget but is dropped.
/// Guarantees: large input can map to small output; explicit event limits and actual output limits remain enforced.
#[test]
fn source_event_size_is_independent_of_output_size() {
    let source = json!({"large": "x".repeat(3 * 1024 * 1024 + 1)});
    for yaml in [
        "max_request_body_size: 8388608\nunmapped: { mode: drop }",
        "max_request_body_size: 8388608\nrules: [{ path: large, target: { kind: drop } }]",
        "max_request_body_size: 8388608\nrules: [{ path: large, target: { kind: drop }, validate: { expected_type: string } }]",
    ] {
        let ep = endpoint(yaml);
        let log = accepted(&ep, source.clone());
        assert!(log.body.is_none());
    }
    let ep = endpoint("max_event_size: 3145728\nunmapped: { mode: drop }");
    assert!(matches!(
        ep.map_event(&source, &MappingContext::default()),
        Err(MappingError::Limit(_))
    ));
    let ep = endpoint("max_request_body_size: 8388608");
    assert!(matches!(
        ep.map_event(&source, &MappingContext::default()),
        Err(MappingError::Limit(_))
    ));
}

/// Scenario: endpoint startup loads exactly 64 routes and then encounters a 65th route, invalid receiver config or malformed YAML.
/// Guarantees: route caps and startup failures are explicit and never return a partial endpoint set.
#[test]
fn endpoint_route_limit_and_load_errors_are_explicit() {
    let dir = tempfile::tempdir().expect("directory");
    std::fs::write(dir.path().join("endpoint.yaml"), "rules: []").expect("endpoint");
    let mut config: Config =
        serde_json::from_value(json!({"listening_addr": "127.0.0.1:4319"})).expect("config");
    for i in 0..64 {
        _ = config.endpoints.insert(
            format!("/route{i}"),
            super::config::EndpointFileRef {
                config_file: "endpoint.yaml".into(),
            },
        );
    }
    assert_eq!(
        load_endpoints(&config, dir.path())
            .expect("64 routes")
            .len(),
        64
    );
    _ = config.endpoints.insert(
        "/overflow".into(),
        super::config::EndpointFileRef {
            config_file: "endpoint.yaml".into(),
        },
    );
    assert!(matches!(
        load_endpoints(&config, dir.path()),
        Err(MappingError::Limit("endpoint count"))
    ));
    _ = config.endpoints.remove("/overflow");
    std::fs::write(dir.path().join("endpoint.yaml"), "rules: [").expect("malformed endpoint");
    assert!(matches!(
        load_endpoints(&config, dir.path()),
        Err(MappingError::Config(_))
    ));
    config.max_request_body_size = 0;
    assert!(matches!(
        load_endpoints(&config, dir.path()),
        Err(MappingError::Config(_))
    ));
}

/// Scenario: an override replaces a required target but omits its required metadata.
/// Guarantees: base defaults remain required and are emitted under the replacement destination.
#[test]
fn target_replacement_preserves_required_defaults() {
    let ep = endpoint(
        r#"
rules:
  - path: name
    target: { kind: event_name }
  - path: id
    target: { kind: attribute, key: original }
    required: { on_missing: default, default_value: generated }
event_overrides:
  Evt:
    rules:
      - path: id
        target: { kind: attribute, key: replacement }
"#,
    );
    let log = accepted(&ep, json!({"name": "Evt"}));
    assert_eq!(log.attributes["replacement"], "generated");
    assert!(!log.attributes.contains_key("original"));
}

/// Scenario: invalid target-specific options, header tokens and protocol normalization inputs are compiled.
/// Guarantees: semantic configuration errors fail before input processing while valid protocol tokens normalize consistently.
#[test]
fn compiler_validates_targets_and_normalizes_protocol_fields() {
    for yaml in [
        "rules: [{ path: name, target: { kind: event_name, split_last: '' } }]",
        "rules: [{ path: name, target: { kind: event_name, strip_segment: { index: 1, match_field: 'tags.*' } } }]",
        "rules: [{ path: name, target: { kind: event_name, strip_segment: { index: 1, match_field: 'tags..key' } } }]",
        "rules: [{ path: name, target: { kind: scope, field: name, split: dash_semver } }]",
        "rules: [{ path: name, target: { kind: scope, field: name_version, split: { last_char: '' } } }]",
        "rules: [{ path: level, target: { kind: severity, map: { high: { number: 25, text: invalid } } } }]",
        "header_mappings: [{ header: 'bad header', target: { kind: attribute } }]",
        "allowed_methods: ['bad method']",
        "accepted_content_types: [invalid]",
        "query_string_headers: { ApiKey: X-Api-Key, apikey: X-Api-Key }",
        "query_string_headers: { key: 'bad header' }",
    ] {
        let config = serde_yaml::from_str(yaml).expect("schema");
        assert!(
            matches!(
                CompiledEndpoint::compile(&config, None, &no_environment),
                Err(MappingError::Config(_))
            ),
            "{yaml}"
        );
    }
    let ep = endpoint(
        r#"
allowed_methods: [post]
accepted_content_types: [" Application/JSON; charset=utf-8 "]
cors: { max_age_seconds: 12, cache_control: "max-age=<max-age>" }
query_string_headers: { ApiKey: X-Api-Key }
"#,
    );
    assert_eq!(ep.config.allowed_methods, ["POST"]);
    assert_eq!(ep.config.accepted_content_types, ["application/json"]);
    assert_eq!(ep.config.cors.cache_control, "max-age=12");
    assert_eq!(ep.config.query_string_headers["apikey"], "x-api-key");
}

/// Scenario: compilation inherits base rules and encounters a very small work budget or invalid zero safeguards.
/// Guarantees: compile-time expansion is budgeted and every zero safeguard is refused rather than disabling its limit.
#[test]
fn compiler_work_budget_and_safeguards_are_enforced() {
    let config: EndpointConfig = serde_yaml::from_str(
        r#"
rules:
  - path: name
    target: { kind: event_name, prefix: "long-prefix." }
  - path: data
    allowed_keys: [long_allowed_key]
event_overrides:
  Evt: {}
"#,
    )
    .expect("config");
    assert!(matches!(
        CompiledEndpoint::with_limits(
            &config,
            None,
            &no_environment,
            Limits {
                work: 32,
                ..Limits::default()
            }
        ),
        Err(MappingError::Limit("traversal work"))
    ));
    for limits in [
        Limits {
            depth: 0,
            ..Limits::default()
        },
        Limits {
            depth: 65,
            ..Limits::default()
        },
        Limits {
            nodes: 0,
            ..Limits::default()
        },
        Limits {
            work: 0,
            ..Limits::default()
        },
        Limits {
            output_bytes: 0,
            ..Limits::default()
        },
        Limits {
            attributes: 0,
            ..Limits::default()
        },
        Limits {
            rules: 0,
            ..Limits::default()
        },
    ] {
        assert!(matches!(
            CompiledEndpoint::with_limits(&config, None, &no_environment, limits),
            Err(MappingError::Config(_))
        ));
    }
}

/// Scenario: a string-coercing rule maps arrays, objects and primitive JSON values.
/// Guarantees: compact JSON is preserved exactly and already-typed numeric and boolean fields remain typed.
#[test]
fn coercion_preserves_json_and_primitive_types() {
    let ep = endpoint(
        r#"
rules:
  - path: coerced
    target: { kind: attribute }
    validate: { expected_type: string_coerce }
  - path: integer
    target: { kind: attribute }
    validate: { expected_type: integer }
  - path: boolean
    target: { kind: attribute }
    validate: { expected_type: boolean }
"#,
    );
    for value in [
        json!({"nested": [1, true]}),
        json!([1, 2]),
        json!(false),
        json!(null),
    ] {
        let log = accepted(
            &ep,
            json!({"coerced": value, "integer": 42, "boolean": true}),
        );
        assert_eq!(log.attributes["coerced"], value.to_string());
        assert_eq!(log.attributes["integer"], 42);
        assert_eq!(log.attributes["boolean"], true);
    }
}

/// Scenario: explicit reject validation receives invalid strings, integers, floats and booleans.
/// Guarantees: all type failures refuse the event rather than emitting partial attributes.
#[test]
fn strict_type_validation_rejects_each_invalid_type() {
    for (expected, value) in [
        ("string", json!(1)),
        ("integer", json!("not an integer")),
        ("float", json!("Infinity")),
        ("boolean", json!("true")),
    ] {
        let ep = endpoint(&format!(
            "rules:\n  - path: value\n    target: {{ kind: attribute }}\n    validate: {{ expected_type: {expected}, on_validation_failure: reject }}"
        ));
        assert!(matches!(
            ep.map_event(&json!({"value": value}), &MappingContext::default()),
            Err(MappingError::Rejected(_))
        ));
    }
}

/// Scenario: separate scope fields and printable event-name validation process valid and invalid inputs.
/// Guarantees: scope name/version map independently and non-printable names or non-string scopes reject explicitly.
#[test]
fn scope_fields_and_printable_name_contracts() {
    let ep = endpoint(
        r#"
event_name_validation: printable_ascii
rules:
  - path: name
    target: { kind: event_name, prefix: "prefix." }
  - path: scope
    target: { kind: scope, field: name }
  - path: version
    target: { kind: scope, field: version }
"#,
    );
    let log = accepted(
        &ep,
        json!({"name": "Event", "scope": "SDK", "version": "1.2.3"}),
    );
    assert_eq!(log.event_name.as_deref(), Some("prefix.Event"));
    assert_eq!(log.scope_name.as_deref(), Some("SDK"));
    assert_eq!(log.scope_version.as_deref(), Some("1.2.3"));
    for input in [
        json!({"name": "space here"}),
        json!({"name": "\n"}),
        json!({"scope": 1}),
    ] {
        assert!(matches!(
            ep.map_event(&input, &MappingContext::default()),
            Err(MappingError::Rejected(_))
        ));
    }
}

/// Scenario: top-level input is not an object, or container-only validation receives a scalar.
/// Guarantees: invalid event shape and invalid allowed-key containers are explicit refusals.
#[test]
fn invalid_event_and_container_shapes_are_refused() {
    let ep = endpoint("rules: [{ path: container, allowed_keys: [allowed] }]");
    for input in [
        json!(null),
        json!([]),
        json!("string"),
        json!({"container": 1}),
    ] {
        assert!(matches!(
            ep.map_event(&input, &MappingContext::default()),
            Err(MappingError::Rejected(_))
        ));
    }
}

/// Scenario: unmatched fields, semantic severity mapping and secondary-only targets encounter less common scalar cases.
/// Guarantees: unknown scalar severities use unspecified, structured severities refuse input and secondary-only rules consume fields.
#[test]
fn severity_scalars_and_secondary_only_rules() {
    let ep = endpoint(
        r#"
rules:
  - path: severity
    target:
      kind: severity
      map:
        "true": { number: 9, text: INFO }
  - path: secondary
    also:
      - target: { kind: resource, key: secondary }
"#,
    );
    assert_eq!(
        accepted(&ep, json!({"severity": true})).severity_number,
        Some(9)
    );
    assert_eq!(
        accepted(&ep, json!({"severity": "unknown"})).severity_number,
        Some(0)
    );
    assert!(matches!(
        ep.map_event(&json!({"severity": {}}), &MappingContext::default()),
        Err(MappingError::Rejected(_))
    ));
    let log = accepted(&ep, json!({"secondary": "retained"}));
    assert_eq!(log.resource["secondary"], "retained");
    assert!(log.body.is_none());
}

/// Scenario: an already-compiled endpoint uses a tiny unmapped-path budget.
/// Guarantees: paths-only mode refuses excess paths before returning an accepted log.
#[test]
fn unmapped_path_collection_is_bounded() {
    let config = serde_yaml::from_str("unmapped: { mode: attribute }").expect("config");
    let ep = CompiledEndpoint::with_limits(
        &config,
        None,
        &no_environment,
        Limits {
            attributes: 1,
            ..Limits::default()
        },
    )
    .expect("compile");
    assert_eq!(
        ep.map_event(&json!({"a": 1, "b": 2}), &MappingContext::default()),
        Err(MappingError::Limit("unmapped paths"))
    );
}

fn endpoint(yaml: &str) -> CompiledEndpoint {
    let config: EndpointConfig = serde_yaml::from_str(yaml).expect("endpoint schema");
    CompiledEndpoint::compile(&config, None, &no_environment).expect("compile endpoint")
}

fn accepted(endpoint: &CompiledEndpoint, event: Value) -> Box<MappedLog> {
    match endpoint
        .map_event(&event, &MappingContext::default())
        .expect("map event")
    {
        MappingOutcome::Accepted(log) => log,
        other => panic!("expected acceptance, got {other:?}"),
    }
}

/// Scenario: each receiver-local endpoint example is compiled with its existing schema.
/// Guarantees: expanded browser, OneCollector and preserving configurations are executable by the mapper.
#[test]
fn all_shipped_endpoints_compile() {
    for yaml in [
        include_str!("azure-monitor-browser-events.yaml"),
        include_str!("onecollector-1ds-endpoint.yaml"),
        include_str!("azure-monitor-envelope.yaml"),
    ] {
        let compiled = endpoint(yaml);
        assert!(!compiled.rules.is_empty());
        assert!(compiled.config.cors.cache_control.contains("3600"));
        assert!(!compiled.config.cors.cache_control.contains("<max-age>"));
    }
}

/// Scenario: a preserving endpoint maps a complete envelope with nested tags, data and extensions.
/// Guarantees: container contents survive as JSON strings, scalar types survive, and unknown structure stays in the body.
#[test]
fn preserving_mapping_keeps_envelope_contents() {
    let ep = endpoint(include_str!("azure-monitor-envelope.yaml"));
    let source = json!({
        "name": "Microsoft.ApplicationInsights.Event", "time": "2024-01-01T00:00:00Z",
        "iKey": "key", "ver": 1, "sampleRate": 50.5,
        "tags": {"ai.user.id": "user"}, "data": {"array": [1, {"text": "quoted \" text"}]},
        "ext": {"arbitrary": true}, "unknown": {"array": [1, 2]}
    });
    let log = accepted(&ep, source.clone());
    assert_eq!(
        log.event_name.as_deref(),
        Some("Microsoft.ApplicationInsights.Event")
    );
    assert_eq!(log.time_unix_nano, Some(1_704_067_200_000_000_000));
    for field in ["tags", "data", "ext"] {
        let encoded = log.attributes[&format!("azure_monitor.envelope.{field}")]
            .as_str()
            .expect("JSON string");
        assert_eq!(
            serde_json::from_str::<Value>(encoded).expect("container JSON"),
            source[field]
        );
    }
    assert_eq!(log.attributes["azure_monitor.envelope.sampleRate"], 50.5);
    assert_eq!(log.body, Some(json!({"unknown": {"array": [1, 2]}})));
}

/// Scenario: a browser exception includes a dashless embedded iKey, dotted tag keys and structured exception details.
/// Guarantees: normalized name, scope/resource attributes, severity and unhandled exception arrays all survive.
#[test]
fn browser_mapping_extracts_resources_scope_and_preserves_exceptions() {
    let ep = endpoint(include_str!("azure-monitor-browser-events.yaml"));
    let source = json!({
        "name": "Microsoft.ApplicationInsights.B71709272D1C44F1ABCD0123456789AB.Exception",
        "iKey": "b7170927-2d1c-44f1-abcd-0123456789ab",
        "time": "2024-01-01T00:00:00Z",
        "tags": {"ai.internal.sdkVersion": "javascript:3.4.1", "ai.user.id": "u", "ai.session.id": "s"},
        "data": {"baseType": "ExceptionData", "baseData": {
            "severityLevel": 3, "exceptions": [{"typeName": "Error", "parsedStack": [{"line": 12}]}]
        }}
    });
    let log = accepted(&ep, source.clone());
    assert_eq!(
        log.event_name.as_deref(),
        Some("Microsoft.ApplicationInsights.Exception")
    );
    assert_eq!(log.scope_name.as_deref(), Some("javascript"));
    assert_eq!(log.scope_version.as_deref(), Some("3.4.1"));
    assert_eq!(log.resource["enduser.pseudo.id"], "u");
    assert_eq!(log.resource["session.id"], "s");
    assert_eq!(log.resource["telemetry.sdk.version"], "3.4.1");
    assert_eq!(log.severity_number, Some(17));
    assert_eq!(log.severity_text.as_deref(), Some("ERROR"));
    assert_eq!(log.attributes["ai.baseData.severityLevel"], 3);
    assert_eq!(
        log.body.as_ref().expect("body")["data"]["baseData"]["exceptions"],
        source["data"]["baseData"]["exceptions"]
    );
}

/// Scenario: Common Schema input carries distributed tracing and an unlisted extension key.
/// Guarantees: known fields map to typed identifiers, while both nested and flattened invalid children reject the event.
#[test]
fn onecollector_trace_context_and_allowed_keys() {
    let ep = endpoint(include_str!("onecollector-1ds-endpoint.yaml"));
    let event = json!({"name": "Test.Event", "ext": {"dt": {
        "traceId": "01010101010101010101010101010101", "spanId": "aBaBaBaBaBaBaBaB", "traceFlags": "1"
    }}});
    let log = accepted(&ep, event.clone());
    assert_eq!(log.trace_id, Some([1; 16]));
    assert_eq!(log.span_id, Some([0xab; 8]));
    assert_eq!(log.flags, Some(1));
    for invalid in [
        json!({"name": "Test.Event", "ext": {"dt": {"unknown": 1}}}),
        json!({"name": "Test.Event", "ext.dt.unknown": 1}),
    ] {
        assert!(matches!(
            ep.map_event(&invalid, &MappingContext::default()),
            Err(MappingError::Rejected(_))
        ));
    }
}

/// Scenario: exact and overlapping glob overrides replace and overlay nested base rules.
/// Guarantees: exact wins, then longest prefix; metadata overlays preserve targets and inheritance can be disabled.
#[test]
fn overrides_are_deterministic_and_merge_flattened_paths() {
    let ep = endpoint(
        r#"
rules:
  - path: name
    target: { kind: event_name }
  - path: data
    children:
      - path: value
        target: { kind: attribute, key: base }
event_overrides:
  "*":
    rules:
      - path: data.value
        target: { kind: attribute, key: catchall }
  "App.*":
    rules:
      - path: data.value
        target: { kind: attribute, key: prefix }
  "App.Exact":
    rules:
      - path: data.value
        required: { on_missing: default, default_value: 7 }
  "Independent":
    inherit_rules: false
    rules:
      - path: data.value
        target: { kind: resource, key: independent }
"#,
    );
    let exact = accepted(&ep, json!({"name": "App.Exact"}));
    assert_eq!(exact.attributes["base"], 7);
    assert!(!exact.attributes.contains_key("prefix"));
    let glob = accepted(&ep, json!({"name": "App.Other", "data": {"value": 3}}));
    assert_eq!(glob.attributes["prefix"], 3);
    let independent = accepted(&ep, json!({"name": "Independent", "data": {"value": 4}}));
    assert_eq!(independent.resource["independent"], 4);
    assert!(independent.event_name.is_none());
    assert_eq!(independent.body, Some(json!({"name": "Independent"})));
}

/// Scenario: rules overlap a wildcard and exact path and collide on destination keys.
/// Guarantees: first matching rule wins per source field; later lexicographic source fields win destination collisions.
#[test]
fn declaration_order_and_attribute_namespaces_are_stable() {
    let ep = endpoint(
        r#"
rules:
  - path: data.*
    target: { kind: attribute, key: value }
  - path: data.a
    target: { kind: resource, key: value }
  - path: resource
    target: { kind: resource, key: value }
"#,
    );
    let log = accepted(&ep, json!({"data": {"z": 2, "a": 1}, "resource": "r"}));
    assert_eq!(log.attributes["value"], 2);
    assert_eq!(log.resource["value"], "r");
}

/// Scenario: environment, header, and event resource mappings target the same attribute.
/// Guarantees: event values override headers, headers override environment, and missing environment variables are omitted.
#[test]
fn resource_precedence_and_environment_resolution() {
    let config: EndpointConfig = serde_yaml::from_str(
        r#"
env_resource_attrs:
  PRESENT: service.name
  MISSING: missing
header_mappings:
  - header: X-Service
    target: { kind: resource, key: service.name }
rules:
  - path: service
    target: { kind: resource, key: service.name }
"#,
    )
    .expect("config");
    let ep = CompiledEndpoint::compile(&config, None, &|name| match name {
        "PRESENT" => Ok("env".into()),
        _ => no_environment(name),
    })
    .expect("compile");
    let context = MappingContext {
        headers: BTreeMap::from([("x-SERVICE".into(), "header".into())]),
        origin: None,
    };
    let MappingOutcome::Accepted(log) = ep.map_event(&json!({}), &context).expect("header") else {
        panic!("accepted");
    };
    assert_eq!(log.resource["service.name"], "header");
    let MappingOutcome::Accepted(log) = ep
        .map_event(&json!({"service": "event"}), &context)
        .expect("event")
    else {
        panic!("accepted");
    };
    assert_eq!(log.resource["service.name"], "event");
    assert!(!log.resource.contains_key("missing"));
}

/// Scenario: a filter remaps accepted keys and separately ignores, rejects, or refuses origins.
/// Guarantees: original GUID stripping precedes remapping and every filter outcome remains distinguishable.
#[test]
fn ikey_filtering_precedes_mapping_and_applies_remapping() {
    let ep = endpoint(
        r#"
rules:
  - path: name
    target: { kind: event_name, strip_segment: { index: 2, match_field: iKey } }
  - path: iKey
    target: { kind: resource, key: ms.iKey }
header_mappings:
  - header: apikey
    target: { kind: attribute, key: header.key }
ikey_filter:
  keys:
    old: { action: accept, remap_ikey: new }
    ignore: { action: ignore }
    reject: { action: reject }
    origin: { action: accept, require_origin: true, allowed_origins: ["https://example.com"] }
"#,
    );
    let log = accepted(&ep, json!({"name": "M.AI.old.Event", "iKey": "old"}));
    assert_eq!(log.event_name.as_deref(), Some("M.AI.Event"));
    assert_eq!(log.resource["ms.iKey"], "new");
    let context = MappingContext {
        headers: BTreeMap::from([("APIKEY".into(), "old".into())]),
        origin: None,
    };
    let MappingOutcome::Accepted(log) = ep.map_event(&json!({}), &context).expect("header key")
    else {
        panic!("accepted");
    };
    assert_eq!(log.attributes["header.key"], "new");
    assert_eq!(
        ep.map_event(&json!({"iKey": "ignore"}), &MappingContext::default()),
        Ok(MappingOutcome::Dropped(DropReason::InstrumentationKey))
    );
    assert!(matches!(
        ep.map_event(&json!({"iKey": "reject"}), &MappingContext::default()),
        Err(MappingError::Rejected(_))
    ));
    assert_eq!(
        ep.map_event(&json!({"iKey": "origin"}), &MappingContext::default()),
        Err(MappingError::Unauthorized)
    );
}

/// Scenario: defaults contain nested objects, while another required rule has no default.
/// Guarantees: default containers pass through ordinary child mapping and a missing required field rejects atomically.
#[test]
fn required_defaults_are_validated_and_mapped_as_normal_input() {
    let ep = endpoint(
        r#"
rules:
  - path: data
    required: { on_missing: default, default_value: { value: "12", other: [1, 2] } }
    children:
      - path: value
        target: { kind: attribute, key: value }
        validate: { expected_type: integer }
  - path: required
    target: { kind: attribute }
    required: { on_missing: reject }
"#,
    );
    let log = accepted(&ep, json!({"required": true}));
    assert_eq!(log.attributes["value"], 12);
    assert_eq!(log.body, Some(json!({"data": {"other": [1, 2]}})));
    assert!(matches!(
        ep.map_event(&json!({}), &MappingContext::default()),
        Err(MappingError::Rejected(_))
    ));
}

/// Scenario: fields require numeric coercion, UTF-8 truncation, skipping and strict length validation.
/// Guarantees: conversions are finite, truncation respects characters, skipped fields disappear and reject remains event-wide.
#[test]
fn validation_coercion_and_utf8_boundaries() {
    let ep = endpoint(
        r#"
rules:
  - path: integer
    target: { kind: attribute }
    validate: { expected_type: integer }
  - path: float
    target: { kind: attribute }
    validate: { expected_type: float }
  - path: text
    target: { kind: attribute }
    validate: { expected_type: string, max_length: 3 }
  - path: skip
    target: { kind: attribute }
    validate: { expected_type: boolean }
  - path: strict
    target: { kind: attribute }
    validate: { expected_type: string, max_length: 2, on_length_exceeded: reject }
"#,
    );
    let log = accepted(
        &ep,
        json!({"integer": "-12", "float": "2.5", "text": "\u{00e9}\u{00e9}", "skip": 1}),
    );
    assert_eq!(log.attributes["integer"], -12);
    assert_eq!(log.attributes["float"], 2.5);
    assert_eq!(log.attributes["text"], "\u{00e9}");
    assert!(!log.attributes.contains_key("skip"));
    assert!(accepted(&ep, json!({"float": "NaN"})).attributes.is_empty());
    assert!(matches!(
        ep.map_event(&json!({"strict": "long"}), &MappingContext::default()),
        Err(MappingError::Rejected(_))
    ));
}

/// Scenario: fan-out combines split, format and SDK semver transforms with scope extraction.
/// Guarantees: every transform receives the validated source, and semver names containing dashes are preserved.
#[test]
fn also_transforms_and_scope_splitting() {
    let ep = endpoint(
        r#"
rules:
  - path: sdk
    target: { kind: scope, field: name_version, split: dash_semver }
    also:
      - target: { kind: attribute, key: name }
        transform: { kind: dash_semver_name }
      - target: { kind: attribute, key: version }
        transform: { kind: dash_semver_version }
      - target: { kind: attribute, key: tail }
        transform: { kind: split_take, delimiter: "-", index: -1 }
      - target: { kind: attribute, key: formatted }
        transform: { kind: format, template: "sdk:{value}" }
"#,
    );
    let log = accepted(&ep, json!({"sdk": "1DS-Web-JS-4.4.1"}));
    assert_eq!(log.scope_name.as_deref(), Some("1DS-Web-JS"));
    assert_eq!(log.scope_version.as_deref(), Some("4.4.1"));
    assert_eq!(log.attributes["name"], "1DS-Web-JS");
    assert_eq!(log.attributes["version"], "4.4.1");
    assert_eq!(log.attributes["tail"], "4.4.1");
    assert_eq!(log.attributes["formatted"], "sdk:1DS-Web-JS-4.4.1");
}

/// Scenario: mapped fields, drops, skipped fields and unhandled nested structures coexist.
/// Guarantees: retained-source size excludes drops/skips/defaults, and the body preserves unhandled nesting without duplication.
#[test]
fn unmapped_body_and_filtered_size_keep_source_structure() {
    let ep = endpoint(
        r#"
filtered_item_size: true
rules:
  - path: mapped
    target: { kind: attribute }
  - path: dropped
    target: { kind: drop }
  - path: skipped
    target: { kind: attribute }
    validate: { expected_type: string }
  - path: container
    children:
      - path: known
        target: { kind: attribute }
  - path: default
    target: { kind: attribute }
    required: { on_missing: default, default_value: "synthetic" }
"#,
    );
    let source = json!({"mapped": "value", "dropped": {"secret": true}, "skipped": 3, "container": {"known": 1, "unknown": [1, {"deep": true}]}, "empty": {}});
    let log = accepted(&ep, source);
    let retained = json!({"mapped": "value", "container": {"known": 1, "unknown": [1, {"deep": true}]}, "empty": {}});
    assert_eq!(
        log.attributes["filtered_item_size"],
        json_size(&retained, usize::MAX).expect("size")
    );
    assert_eq!(
        log.body,
        Some(json!({"container": {"unknown": [1, {"deep": true}]}, "empty": {}}))
    );
}

/// Scenario: unmapped fields are configured as paths-only or dropped entirely.
/// Guarantees: paths are deterministic, nested arrays are treated as single fields and no source values leak into the body.
#[test]
fn unmapped_paths_and_drop_modes() {
    let ep = endpoint("unmapped: { mode: attribute, attribute_key: unknown }");
    let log = accepted(&ep, json!({"z": [1, 2], "a": {"b": true}}));
    assert_eq!(log.attributes["unknown"], json!(["a.b", "z"]));
    assert!(log.body.is_none());
    let log = accepted(&endpoint("unmapped: { mode: drop }"), json!({"secret": 1}));
    assert!(log.body.is_none());
    assert!(log.attributes.is_empty());
}

/// Scenario: event overrides and post-transform drop patterns select different outcomes.
/// Guarantees: override rejection is request-wide, drops are explicit and final name validation runs on transformed names.
#[test]
fn event_actions_and_name_validation() {
    let ep = endpoint(
        r#"
event_name_validation: otel_semconv
drop_events: ["internal.*"]
rules:
  - path: name
    target: { kind: event_name, split_last: ":" }
event_overrides:
  rejected: { action: reject }
  dropped: { action: drop }
"#,
    );
    assert_eq!(
        ep.map_event(&json!({"name": "raw:rejected"}), &MappingContext::default()),
        Err(MappingError::RejectedPayload)
    );
    assert_eq!(
        ep.map_event(&json!({"name": "raw:dropped"}), &MappingContext::default()),
        Ok(MappingOutcome::Dropped(DropReason::Override))
    );
    assert_eq!(
        ep.map_event(
            &json!({"name": "raw:internal.event"}),
            &MappingContext::default()
        ),
        Ok(MappingOutcome::Dropped(DropReason::EventName))
    );
    for name in ["", "Upper", "a..b", "a_", "a b"] {
        assert!(
            matches!(
                ep.map_event(&json!({"name": name}), &MappingContext::default()),
                Err(MappingError::Rejected(_))
            ),
            "{name}"
        );
    }
    assert_eq!(
        accepted(&ep, json!({"name": "raw:valid.event_2"}))
            .event_name
            .as_deref(),
        Some("valid.event_2")
    );
}

/// Scenario: malformed timestamps, trace IDs and flags are supplied without explicit validation.
/// Guarantees: semantic conversion errors reject the event instead of silently erasing or corrupting OTel fields.
#[test]
fn invalid_semantic_fields_fail_explicitly_without_panics() {
    for (kind, values) in [
        (
            "trace_id",
            vec![json!("\u{00e9}".repeat(16)), json!("xyz"), json!(1)],
        ),
        ("span_id", vec![json!("z".repeat(16)), json!("00")]),
        (
            "timestamp",
            vec![
                json!("not time"),
                json!("1960-01-01T00:00:00Z"),
                json!("9999-01-01T00:00:00Z"),
            ],
        ),
        ("trace_flags", vec![json!(-1), json!(u64::MAX), json!("no")]),
    ] {
        let ep = endpoint(&format!(
            "rules:\n  - path: value\n    target: {{ kind: {kind} }}"
        ));
        for value in values {
            assert!(matches!(
                ep.map_event(&json!({"value": value}), &MappingContext::default()),
                Err(MappingError::Rejected(_))
            ));
        }
    }
}

/// Scenario: invalid rule paths, patterns, targets and missing defaults are compiled.
/// Guarantees: unusable configuration fails before processing input rather than selecting silent fallback behavior.
#[test]
fn invalid_endpoint_contracts_fail_compilation() {
    for yaml in [
        "body_format: []",
        "max_event_size: 0",
        "event_overrides: { 'a*b': {} }",
        "rules: [{ path: 'a..b', target: { kind: drop } }]",
        "rules: [{ path: 'a.*', required: {} }]",
        "rules: [{ path: a, required: { on_missing: default } }]",
        "rules: [{ path: a, target: { kind: attribute, key: '' } }]",
        "rules: [{ path: a, also: [{ target: { kind: attribute }, transform: { kind: split_take, delimiter: '' } }] }]",
        "header_mappings: [{ header: x, target: { kind: timestamp } }]",
        "env_resource_attrs: { A: duplicate, B: duplicate }",
    ] {
        let config = serde_yaml::from_str(yaml).expect("schema accepts contract");
        assert!(
            CompiledEndpoint::compile(&config, None, &no_environment).is_err(),
            "{yaml}"
        );
    }
}

/// Scenario: exact byte, node, depth, work and attribute thresholds are crossed.
/// Guarantees: each safeguard is independently enforced, including during output fan-out, with no partial accepted log.
#[test]
fn mapping_limits_are_enforced_at_boundaries() {
    let ep = endpoint("{}");
    let source = json!({"a": "x"});
    let size = json_size(&source, usize::MAX).expect("size");
    let mut config = ep.config.clone();
    config.max_event_size = Some(size);
    assert!(
        CompiledEndpoint::compile(&config, None, &no_environment)
            .expect("compile")
            .map_event(&source, &MappingContext::default())
            .is_ok()
    );
    config.max_event_size = Some(size - 1);
    assert!(matches!(
        CompiledEndpoint::compile(&config, None, &no_environment)
            .expect("compile")
            .map_event(&source, &MappingContext::default()),
        Err(MappingError::Limit(_))
    ));

    let limits = Limits {
        nodes: 2,
        depth: 2,
        ..Limits::default()
    };
    assert!(inspect_json(&source, limits, &mut Budget::new(100)).is_ok());
    assert_eq!(
        inspect_json(&json!({"a": "x", "b": 2}), limits, &mut Budget::new(100)),
        Err(DataLimitError::Limit("value nodes"))
    );
    assert_eq!(
        inspect_json(&json!({"a": {"b": 1}}), limits, &mut Budget::new(100)),
        Err(DataLimitError::Limit("nesting depth"))
    );
    let mut budget = Budget::new(3);
    assert!(budget.charge(3).is_ok());
    assert_eq!(
        budget.charge(1),
        Err(DataLimitError::Limit("traversal work"))
    );

    let config = serde_yaml::from_str("rules: [{ path: '*', target: { kind: attribute } }]")
        .expect("config");
    let ep = CompiledEndpoint::with_limits(
        &config,
        None,
        &no_environment,
        Limits {
            attributes: 1,
            ..Limits::default()
        },
    )
    .expect("compile");
    assert_eq!(accepted(&ep, json!({"a": 1})).attributes.len(), 1);
    assert_eq!(
        ep.map_event(&json!({"a": 1, "b": 2}), &MappingContext::default()),
        Err(MappingError::Limit("attribute count"))
    );

    let ep = CompiledEndpoint::with_limits(
        &config,
        None,
        &no_environment,
        Limits {
            output_bytes: 512,
            ..Limits::default()
        },
    )
    .expect("compile");
    assert!(
        ep.map_event(&json!({"a": "x".repeat(512)}), &MappingContext::default())
            .is_err()
    );
    assert_eq!(json_size(&json!("\""), 4), Ok(4));
    assert_eq!(
        json_size(&json!("\""), 3),
        Err(DataLimitError::Limit("serialized bytes"))
    );
}

/// Scenario: an accepted field fans out enough data to exceed the cumulative output budget.
/// Guarantees: mapping fails even though the source event fits its input limit, and an empty subsequent event still maps independently.
#[test]
fn fanout_is_bounded_and_failures_do_not_mutate_compiled_endpoints() {
    let config = serde_yaml::from_str(
        r#"
rules:
  - path: value
    target: { kind: attribute, key: first }
    also:
      - target: { kind: attribute, key: second }
      - target: { kind: attribute, key: third }
"#,
    )
    .expect("config");
    let ep = CompiledEndpoint::with_limits(
        &config,
        None,
        &no_environment,
        Limits {
            output_bytes: 512,
            ..Limits::default()
        },
    )
    .expect("compile");
    assert!(matches!(
        ep.map_event(
            &json!({"value": "x".repeat(100)}),
            &MappingContext::default()
        ),
        Err(MappingError::Limit(_))
    ));
    assert!(accepted(&ep, json!({})).attributes.is_empty());
}

/// Scenario: endpoint references load from a pipeline directory and one route is invalid.
/// Guarantees: paths are confined, endpoint byte caps are bounded, and startup never returns a partial route set.
#[test]
fn endpoint_file_loading_is_confined_bounded_and_atomic() {
    let dir = tempfile::tempdir().expect("directory");
    std::fs::write(dir.path().join("endpoint.yaml"), "rules: []").expect("endpoint");
    let mut config: Config = serde_json::from_value(json!({
        "listening_addr": "127.0.0.1:4319", "endpoints": { "/ingest": { "config_file": "endpoint.yaml" } }
    })).expect("config");
    assert_eq!(load_endpoints(&config, dir.path()).expect("load").len(), 1);
    config
        .endpoints
        .get_mut("/ingest")
        .expect("route")
        .config_file = "../escape.yaml".into();
    assert!(matches!(
        load_endpoints(&config, dir.path()),
        Err(MappingError::File(_))
    ));
    config
        .endpoints
        .get_mut("/ingest")
        .expect("route")
        .config_file = "endpoint.yaml".into();
    std::fs::write(
        dir.path().join("endpoint.yaml"),
        "max_request_body_size: 99999999",
    )
    .expect("invalid endpoint");
    assert!(matches!(
        load_endpoints(&config, dir.path()),
        Err(MappingError::Config(_))
    ));
    std::fs::write(
        dir.path().join("endpoint.yaml"),
        vec![b' '; CONFIG_FILE_BYTES],
    )
    .expect("at limit");
    assert_eq!(
        read_config(&dir.path().join("endpoint.yaml"))
            .expect("read")
            .len(),
        CONFIG_FILE_BYTES
    );
    std::fs::write(
        dir.path().join("endpoint.yaml"),
        vec![b' '; CONFIG_FILE_BYTES + 1],
    )
    .expect("over limit");
    assert_eq!(
        read_config(&dir.path().join("endpoint.yaml")),
        Err(DataLimitError::Limit("configuration file bytes"))
    );
}

/// Scenario: flattened container keys reach wildcard targets and dropped parents have explicitly mapped children.
/// Guarantees: wildcard suffixes retain literal dotted names, while dropped descendants neither leak nor inflate retained size.
#[test]
fn flattened_wildcard_suffixes_and_explicit_children_of_drops() {
    let ep = endpoint(
        r#"
filtered_item_size: true
rules:
  - path: data.properties.*
    target: { kind: attribute, prefix: "property." }
  - path: secret
    target: { kind: drop }
    children:
      - path: visible
        target: { kind: attribute, key: visible }
"#,
    );
    let log = accepted(
        &ep,
        json!({
            "data.properties": { "literal.dotted": 1 },
            "secret": { "hidden": "not retained", "visible": true }
        }),
    );
    assert_eq!(log.attributes["property.literal.dotted"], 1);
    assert_eq!(log.attributes["visible"], true);
    assert!(log.body.is_none());
    let retained = json!({"data.properties": {"literal.dotted": 1}, "secret": {"visible": true}});
    assert_eq!(
        log.attributes["filtered_item_size"],
        json_size(&retained, usize::MAX).expect("size")
    );
}

/// Scenario: compiled override expansion reaches its exact rule limit and rule depth reaches its configured boundary.
/// Guarantees: replacement overrides do not consume extra rule slots, and the next expanded rule or nesting level fails.
#[test]
fn compilation_rule_and_depth_limits_are_exact() {
    let config: EndpointConfig = serde_yaml::from_str(
        r#"
rules:
  - path: name
    target: { kind: event_name }
event_overrides:
  exact:
    rules:
      - path: name
        target: { kind: event_name, prefix: "prefix." }
"#,
    )
    .expect("config");
    assert!(
        CompiledEndpoint::with_limits(
            &config,
            None,
            &no_environment,
            Limits {
                rules: 2,
                ..Limits::default()
            }
        )
        .is_ok()
    );
    assert!(matches!(
        CompiledEndpoint::with_limits(
            &config,
            None,
            &no_environment,
            Limits {
                rules: 1,
                ..Limits::default()
            }
        ),
        Err(MappingError::Limit("compiled rules"))
    ));
    let config: EndpointConfig =
        serde_yaml::from_str("rules: [{ path: a.b, target: { kind: drop } }]").expect("config");
    assert!(
        CompiledEndpoint::with_limits(
            &config,
            None,
            &no_environment,
            Limits {
                depth: 2,
                ..Limits::default()
            }
        )
        .is_ok()
    );
    assert!(matches!(
        CompiledEndpoint::with_limits(
            &config,
            None,
            &no_environment,
            Limits {
                depth: 1,
                ..Limits::default()
            }
        ),
        Err(MappingError::Limit("rule depth"))
    ));
}

/// Scenario: mapping work and input depth limits are exceeded even though output is dropped.
/// Guarantees: dropping data cannot bypass inspection safeguards, and repeated mappings do not share exhausted budgets.
#[test]
fn work_and_depth_limits_apply_before_cloning_or_dropping() {
    let config: EndpointConfig = serde_yaml::from_str("unmapped: { mode: drop }").expect("config");
    let ep = CompiledEndpoint::with_limits(
        &config,
        None,
        &no_environment,
        Limits {
            work: 32,
            ..Limits::default()
        },
    )
    .expect("compile");
    assert!(matches!(
        ep.map_event(
            &json!({"value": "x".repeat(40)}),
            &MappingContext::default()
        ),
        Err(MappingError::Limit("traversal work"))
    ));
    let ep = CompiledEndpoint::with_limits(
        &config,
        None,
        &no_environment,
        Limits {
            depth: 2,
            ..Limits::default()
        },
    )
    .expect("compile");
    assert!(matches!(
        ep.map_event(&json!({"a": [1]}), &MappingContext::default()),
        Err(MappingError::Limit("nesting depth"))
    ));
    assert!(
        ep.map_event(&json!({"a": 1}), &MappingContext::default())
            .is_ok()
    );
}

/// Scenario: a configured additional-target transform expands excessively or cannot select a string part.
/// Guarantees: oversized expansion and invalid transforms fail atomically rather than emitting only the primary target.
#[test]
fn transform_expansion_and_invalid_selection_are_explicit_failures() {
    let config: EndpointConfig = serde_yaml::from_str(
        r#"
rules:
  - path: value
    target: { kind: attribute, key: original }
    also:
      - target: { kind: attribute, key: expanded }
        transform: { kind: format, template: "{value}{value}{value}{value}{value}{value}" }
"#,
    )
    .expect("config");
    let ep = CompiledEndpoint::with_limits(
        &config,
        None,
        &no_environment,
        Limits {
            output_bytes: 512,
            ..Limits::default()
        },
    )
    .expect("compile");
    assert!(matches!(
        ep.map_event(
            &json!({"value": "x".repeat(100)}),
            &MappingContext::default()
        ),
        Err(MappingError::Limit(_))
    ));
    let ep = endpoint(
        r#"
rules:
  - path: value
    target: { kind: attribute }
    also:
      - target: { kind: attribute, key: selected }
        transform: { kind: split_take, delimiter: ".", index: 10 }
"#,
    );
    assert!(matches!(
        ep.map_event(&json!({"value": "one.part"}), &MappingContext::default()),
        Err(MappingError::Rejected(_))
    ));
}

/// Scenario: source headers differ only by case and an environment resolver reports non-Unicode data.
/// Guarantees: ambiguous headers and invalid environment values are surfaced rather than silently selecting or omitting data.
#[test]
fn context_errors_are_not_hidden() {
    let ep = endpoint("{}");
    let context = MappingContext {
        headers: BTreeMap::from([("X-Key".into(), "a".into()), ("x-key".into(), "b".into())]),
        origin: None,
    };
    assert!(matches!(
        ep.map_event(&json!({}), &context),
        Err(MappingError::Rejected(_))
    ));
    let config = serde_yaml::from_str("env_resource_attrs: { ENV: resource }").expect("config");
    assert!(matches!(
        CompiledEndpoint::compile(&config, None, &|_| Err(std::env::VarError::NotUnicode(
            "invalid".into()
        ))),
        Err(MappingError::Config(_))
    ));
}

/// Scenario: a loaded endpoint references an external instrumentation-key filter.
/// Guarantees: the loader compiles the filter with the pipeline directory and mapped resource keys reflect its verdict.
#[test]
fn endpoint_loading_compiles_external_filters() {
    let dir = tempfile::tempdir().expect("directory");
    std::fs::write(
        dir.path().join("endpoint.yaml"),
        r#"
ikey_filter: { filter_file: keys.yaml }
rules:
  - path: iKey
    target: { kind: resource, key: ms.iKey }
"#,
    )
    .expect("endpoint");
    std::fs::write(
        dir.path().join("keys.yaml"),
        "keys: { old: { action: accept, remap_ikey: new } }",
    )
    .expect("filter");
    let config = serde_json::from_value(json!({
        "listening_addr": "127.0.0.1:4319",
        "endpoints": { "/ingest": { "config_file": "endpoint.yaml" } }
    }))
    .expect("config");
    let endpoints = load_endpoints(&config, dir.path()).expect("load");
    assert_eq!(
        accepted(&endpoints["/ingest"], json!({"iKey": "old"})).resource["ms.iKey"],
        "new"
    );
}

/// Scenario: all explicit browser severity values and an unknown value are supplied.
/// Guarantees: numeric lookups preserve each configured severity and unknown values use the documented unspecified fallback.
#[test]
fn severity_lookup_covers_every_explicit_level() {
    let ep = endpoint(include_str!("azure-monitor-browser-events.yaml"));
    for (level, expected) in [(0, 1), (1, 9), (2, 13), (3, 17), (4, 21), (99, 0)] {
        let log = accepted(
            &ep,
            json!({
                "name": "Microsoft.ApplicationInsights.Message",
                "data": {"baseData": {"severityLevel": level, "message": "example"}}
            }),
        );
        assert_eq!(log.severity_number, Some(expected), "{level}");
    }
}

/// Scenario: RFC 3339 timestamps carry subsecond precision, offsets and values beyond signed nanosecond range.
/// Guarantees: both timestamp fields preserve the full unsigned OTLP nanosecond range without timezone or precision loss.
#[test]
fn timestamps_preserve_offsets_precision_and_unsigned_range() {
    let ep = endpoint(
        r#"
rules:
  - path: time
    target: { kind: timestamp }
  - path: observed
    target: { kind: observed_timestamp }
"#,
    );
    let log = accepted(
        &ep,
        json!({
            "time": "1970-01-01T01:00:00.123456789+01:00",
            "observed": "2500-01-01T00:00:00Z"
        }),
    );
    assert_eq!(log.time_unix_nano, Some(123_456_789));
    assert_eq!(
        log.observed_time_unix_nano,
        Some(16_725_225_600_000_000_000)
    );
}

/// Scenario: a mapped container also declares explicit child mappings.
/// Guarantees: parent serialization does not suppress child extraction, and retained source size counts the container only once.
#[test]
fn mapped_containers_do_not_hide_child_rules() {
    let ep = endpoint(
        r#"
filtered_item_size: true
rules:
  - path: data
    target: { kind: attribute, key: original }
    children:
      - path: value
        target: { kind: resource, key: value }
"#,
    );
    let source = json!({"data": {"value": 1, "other": true}});
    let log = accepted(&ep, source.clone());
    assert_eq!(
        serde_json::from_str::<Value>(log.attributes["original"].as_str().expect("JSON string"))
            .expect("data"),
        source["data"]
    );
    assert_eq!(log.resource["value"], 1);
    assert!(log.body.is_none());
    assert_eq!(
        log.attributes["filtered_item_size"],
        json_size(&source, usize::MAX).expect("size")
    );
}

/// Scenario: an unexpanded source item approaches the configured protocol event-size limit.
/// Guarantees: the work safeguard accommodates ordinary megabyte-scale payloads rather than rejecting them solely for inspecting bytes.
#[test]
fn default_work_budget_allows_large_bounded_source_events() {
    let ep = endpoint("max_event_size: 2097152\nunmapped: { mode: drop }");
    let log = accepted(&ep, json!({"value": "x".repeat(1024 * 1024)}));
    assert!(log.body.is_none());
}

/// Scenario: a numeric field validated as float already contains a large JSON integer.
/// Guarantees: accepting a numeric value does not round-trip it through f64 and silently lose integer precision.
#[test]
fn numeric_validation_preserves_existing_integer_precision() {
    let ep = endpoint(
        r#"
rules:
  - path: value
    target: { kind: attribute }
    validate: { expected_type: float }
"#,
    );
    let value = 9_007_199_254_740_993u64;
    assert_eq!(
        accepted(&ep, json!({"value": value})).attributes["value"],
        value
    );
}

/// Scenario: one attribute fits the exact cumulative emitted-byte budget and then loses one byte of capacity.
/// Guarantees: output admission uses a measurable byte boundary rather than checking only the final oversized result.
#[test]
fn cumulative_output_budget_has_an_exact_boundary() {
    let config =
        serde_yaml::from_str("rules: [{ path: a, target: { kind: attribute } }]").expect("config");
    let empty = json_size(&MappedLog::default(), usize::MAX).expect("empty size");
    let entry = json_size(&"a", usize::MAX).expect("key size")
        + json_size(&1, usize::MAX).expect("value size")
        + 2;
    for (limit, success) in [(empty + entry, true), (empty + entry - 1, false)] {
        let ep = CompiledEndpoint::with_limits(
            &config,
            None,
            &no_environment,
            Limits {
                output_bytes: limit,
                ..Limits::default()
            },
        )
        .expect("compile");
        let outcome = ep.map_event(&json!({"a": 1}), &MappingContext::default());
        if success {
            assert!(matches!(outcome, Ok(MappingOutcome::Accepted(_))));
        } else {
            assert_eq!(outcome, Err(MappingError::Limit("output bytes")));
        }
    }
}

/// Scenario: individually valid required defaults together exceed the logical event's node budget.
/// Guarantees: expansion fails during injection instead of constructing all defaults and checking only after mapping.
#[test]
fn defaults_share_the_logical_event_node_budget() {
    let config = serde_yaml::from_str(
        r#"
rules:
  - path: a
    required: { on_missing: default, default_value: [1] }
  - path: b
    required: { on_missing: default, default_value: [2] }
"#,
    )
    .expect("config");
    let ep = CompiledEndpoint::with_limits(
        &config,
        None,
        &no_environment,
        Limits {
            nodes: 3,
            ..Limits::default()
        },
    )
    .expect("compile");
    assert_eq!(
        ep.map_event(&json!({}), &MappingContext::default()),
        Err(MappingError::Limit("value nodes"))
    );
}

/// Scenario: an input contains a dotted key with more virtual path segments than allowed.
/// Guarantees: even a wildcard mapping refuses excessive logical depth before expanding the complete key into a path.
#[test]
fn dotted_keys_obey_the_path_depth_limit() {
    let ep = endpoint("rules: [{ path: '*', target: { kind: attribute } }]");
    let event = json!({ "a.".repeat(100_000): 1 });
    assert_eq!(
        ep.map_event(&event, &MappingContext::default()),
        Err(MappingError::Limit("field path depth"))
    );
}
