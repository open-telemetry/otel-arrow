// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Annotation parsing and validation tests: value specs, attribute types,
//! and signal-level (metric, span, event) annotations.
//!
//! Shared helpers live in the parent module.

use super::*;

// ===========================================================================
// Parsing: value spec forms
// ===========================================================================

/// Scenario: A spec with only `values` is parsed for an int attribute.
/// Guarantees: The discrete form is selected, values keep their order and
/// `weights` stays `None` so sampling is uniform.
#[test]
fn parse_discrete_values_only() {
    let spec = parse_scalar("values: [80, 443, 8080]", ScalarKind::Int).expect("valid");
    assert_eq!(
        spec,
        ValueSpec::Discrete {
            values: vec![Scalar::Int(80), Scalar::Int(443), Scalar::Int(8080)],
            weights: None,
        }
    );
}

/// Scenario: A spec with `values` and matching `weights` is parsed.
/// Guarantees: Weights are preserved as relative f64 values alongside values.
#[test]
fn parse_discrete_weighted() {
    let spec =
        parse_scalar("values: [a, b]\nweights: [3, 1.5]", ScalarKind::String).expect("valid");
    assert_eq!(
        spec,
        ValueSpec::Discrete {
            values: vec![Scalar::String("a".into()), Scalar::String("b".into())],
            weights: Some(vec![3.0, 1.5]),
        }
    );
}

/// Scenario: A spec with only `range` is parsed.
/// Guarantees: The continuous form is selected with a uniform distribution.
#[test]
fn parse_continuous_range_defaults_uniform() {
    let spec = parse_scalar("range: { min: 1024, max: 65535 }", ScalarKind::Int).expect("valid");
    assert_eq!(
        spec,
        ValueSpec::Continuous {
            range: Range {
                min: 1024.0,
                max: 65535.0
            },
            distribution: Distribution::Uniform,
        }
    );
}

/// Scenario: Each distribution type is parsed with its required parameters.
/// Guarantees: Every documented distribution maps to the matching enum
/// variant with parameters in value units.
#[test]
fn parse_every_distribution_type() {
    let cases = [
        (
            "{ type: uniform }",
            Distribution::Uniform,
            ScalarKind::Double,
        ),
        (
            "{ type: normal, mean: 5, stddev: 2 }",
            Distribution::Normal {
                mean: 5.0,
                stddev: 2.0,
            },
            ScalarKind::Double,
        ),
        (
            "{ type: lognormal, mean: 0.2, stddev: 0.5 }",
            Distribution::LogNormal {
                mean: 0.2,
                stddev: 0.5,
            },
            ScalarKind::Double,
        ),
        (
            "{ type: exponential, mean: 3 }",
            Distribution::Exponential { mean: 3.0 },
            ScalarKind::Double,
        ),
        (
            "{ type: zipf, exponent: 1.1 }",
            Distribution::Zipf { exponent: 1.1 },
            ScalarKind::Int,
        ),
    ];
    for (src, expected, kind) in cases {
        let spec = parse_scalar(
            &format!("range: {{ min: 1, max: 10 }}\ndistribution: {src}"),
            kind,
        )
        .unwrap_or_else(|e| panic!("{src}: {e}"));
        match spec {
            ValueSpec::Continuous { distribution, .. } => assert_eq!(distribution, expected),
            other => panic!("{src}: unexpected {other:?}"),
        }
    }
}

/// Scenario: YAML integers are given for a double attribute.
/// Guarantees: They are accepted and converted to doubles.
#[test]
fn parse_double_accepts_integer_literals() {
    let spec = parse_scalar("values: [1, 2.5]", ScalarKind::Double).expect("valid");
    assert_eq!(
        spec,
        ValueSpec::Discrete {
            values: vec![Scalar::Double(1.0), Scalar::Double(2.5)],
            weights: None
        }
    );
}

/// Scenario: An `any` attribute lists mixed scalar values.
/// Guarantees: Each value keeps its own scalar type.
#[test]
fn parse_any_mixed_scalars() {
    let spec = parse_scalar(r#"values: ["abc", 42, 3.5, true]"#, ScalarKind::Any).expect("valid");
    assert_eq!(
        spec,
        ValueSpec::Discrete {
            values: vec![
                Scalar::String("abc".into()),
                Scalar::Int(42),
                Scalar::Double(3.5),
                Scalar::Bool(true)
            ],
            weights: None
        }
    );
}

/// Scenario: A min == max range is parsed with a non-uniform distribution.
/// Guarantees: The degenerate range is accepted (it is a constant).
#[test]
fn parse_degenerate_range_is_valid() {
    let _ = parse_scalar(
        "range: { min: 5, max: 5 }\ndistribution: { type: normal, mean: 0, stddev: 1 }",
        ScalarKind::Double,
    )
    .expect("constant range is valid");
}

// ===========================================================================
// Parsing: rejected value specs
// ===========================================================================

/// Scenario: Each invalid value-spec shape (mutual exclusion, pairing,
/// weights, ranges, distribution parameters, zero-mass ranges) is parsed.
/// Guarantees: Every one is rejected with the expected key path and reason,
/// so annotation mistakes surface at load time rather than silently.
#[test]
fn parse_rejects_invalid_value_specs() {
    let ns = annotation::NAMESPACE;
    let cases: Vec<(&str, ScalarKind, String, &str)> = vec![
        (
            "values: [1, 2]\nrange: { min: 0, max: 5 }",
            ScalarKind::Int,
            ns.into(),
            "mutually exclusive",
        ),
        (
            "weights: [1, 2]",
            ScalarKind::Int,
            ns.into(),
            "requires `values` or `range`",
        ),
        (
            "{}",
            ScalarKind::Int,
            ns.into(),
            "requires `values` or `range`",
        ),
        (
            "values: [1, 2]\ndistribution: { type: uniform }",
            ScalarKind::Int,
            ns.into(),
            "`distribution` requires `range`",
        ),
        (
            "range: { min: 0, max: 5 }\nweights: [1]",
            ScalarKind::Int,
            ns.into(),
            "`weights` requires `values`",
        ),
        (
            "values: [1, 2]\nweights: [1]",
            ScalarKind::Int,
            format!("{ns}.weights"),
            "does not match",
        ),
        (
            "values: [1, 2]\nweights: [0, 0]",
            ScalarKind::Int,
            format!("{ns}.weights"),
            "sum to > 0",
        ),
        (
            "values: [1, 2]\nweights: [1, -1]",
            ScalarKind::Int,
            format!("{ns}.weights[1]"),
            ">= 0",
        ),
        (
            "values: [1, 2]\nweights: [1, x]",
            ScalarKind::Int,
            format!("{ns}.weights[1]"),
            "number",
        ),
        (
            "values: []",
            ScalarKind::Int,
            format!("{ns}.values"),
            "must not be empty",
        ),
        (
            "values: 5",
            ScalarKind::Int,
            format!("{ns}.values"),
            "must be a list",
        ),
        (
            "range: { min: 5, max: 0 }",
            ScalarKind::Int,
            format!("{ns}.range"),
            "must be <= max",
        ),
        (
            "range: { min: 0 }",
            ScalarKind::Int,
            format!("{ns}.range"),
            "missing `max`",
        ),
        (
            "range: [0, 5]",
            ScalarKind::Int,
            format!("{ns}.range"),
            "mapping",
        ),
        (
            "range: { min: 0.5, max: 5 }",
            ScalarKind::Int,
            format!("{ns}.range.min"),
            "integer bounds",
        ),
        (
            // JSON has no representation for infinity, so a YAML `.inf` bound
            // transcodes to null and is rejected as a non-number rather than a
            // non-finite number. Either way the bound is refused.
            "range: { min: 0, max: .inf }",
            ScalarKind::Double,
            format!("{ns}.range.max"),
            "must be a number",
        ),
        (
            "range: { min: 0, max: 5 }\ndistribution: { type: normal, mean: 2 }",
            ScalarKind::Double,
            format!("{ns}.distribution"),
            "missing required `stddev`",
        ),
        (
            "range: { min: 0, max: 5 }\ndistribution: { type: normal, mean: 2, stddev: 0 }",
            ScalarKind::Double,
            format!("{ns}.distribution.stddev"),
            "> 0",
        ),
        (
            "range: { min: 0, max: 5 }\ndistribution: { type: lognormal, mean: -1, stddev: 1 }",
            ScalarKind::Double,
            format!("{ns}.distribution.mean"),
            "> 0",
        ),
        (
            "range: { min: 0, max: 5 }\ndistribution: { type: exponential, mean: 0 }",
            ScalarKind::Double,
            format!("{ns}.distribution.mean"),
            "> 0",
        ),
        (
            "range: { min: 1, max: 5 }\ndistribution: { type: zipf, exponent: -1 }",
            ScalarKind::Int,
            format!("{ns}.distribution.exponent"),
            "> 0",
        ),
        (
            "range: { min: 0, max: 5 }\ndistribution: { type: gamma }",
            ScalarKind::Double,
            format!("{ns}.distribution.type"),
            "unknown distribution",
        ),
        (
            "range: { min: 0, max: 5 }\ndistribution: { mean: 2 }",
            ScalarKind::Double,
            format!("{ns}.distribution"),
            "missing `type`",
        ),
        (
            "range: { min: 0.0, max: 1.0 }\ndistribution: { type: zipf, exponent: 1 }",
            ScalarKind::Double,
            format!("{ns}.distribution.type"),
            "zipf only applies to int",
        ),
        (
            "range: { min: 0, max: 1 }\ndistribution: { type: normal, mean: 100, stddev: 1 }",
            ScalarKind::Double,
            format!("{ns}.distribution"),
            "~0 probability",
        ),
        (
            "range: { min: -5, max: 0 }\ndistribution: { type: lognormal, mean: 1, stddev: 1 }",
            ScalarKind::Double,
            format!("{ns}.distribution"),
            "range.max > 0",
        ),
        (
            "range: { min: -5, max: -1 }\ndistribution: { type: exponential, mean: 1 }",
            ScalarKind::Double,
            format!("{ns}.distribution"),
            "range.max >= 0",
        ),
        (
            "range: { min: 0, max: 5 }",
            ScalarKind::String,
            format!("{ns}.range"),
            "continuous form is not allowed for string",
        ),
        (
            "range: { min: 0, max: 1 }",
            ScalarKind::Bool,
            format!("{ns}.range"),
            "continuous form is not allowed for boolean",
        ),
        (
            "range: { min: 0, max: 1 }",
            ScalarKind::Any,
            format!("{ns}.range"),
            "continuous form is not allowed for any",
        ),
        (
            r#"values: ["a"]"#,
            ScalarKind::Int,
            format!("{ns}.values[0]"),
            "does not match attribute type int",
        ),
        (
            "values: [1.5]",
            ScalarKind::Int,
            format!("{ns}.values[0]"),
            "does not match attribute type int",
        ),
        (
            "values: [1]",
            ScalarKind::Bool,
            format!("{ns}.values[0]"),
            "does not match attribute type boolean",
        ),
        (
            "values: [true]",
            ScalarKind::String,
            format!("{ns}.values[0]"),
            "does not match attribute type string",
        ),
        (
            "values: [[1]]",
            ScalarKind::Any,
            format!("{ns}.values[0]"),
            "does not match attribute type any",
        ),
    ];
    for (src, kind, path, needle) in cases {
        let result = parse_scalar(src, kind);
        match &result {
            Err(e) if e.path == path && e.message.contains(needle) => {}
            _ => panic!("case {src:?} ({kind:?}): expected {path}: {needle}, got {result:?}"),
        }
    }
}

/// Scenario: A spec contains an unknown or misspelled key.
/// Guarantees: The key is rejected (for example `weight` instead of
/// `weights`) instead of silently sampling uniformly.
#[test]
fn parse_rejects_unknown_keys() {
    let ns = annotation::NAMESPACE;
    expect_err(
        parse_scalar("values: [1, 2]\nweight: [1, 1]", ScalarKind::Int),
        ns,
        "unknown key \"weight\"",
    );
    expect_err(
        parse_scalar("range: { min: 0, max: 5, step: 1 }", ScalarKind::Int),
        &format!("{ns}.range"),
        "unknown key \"step\"",
    );
    expect_err(
        parse_scalar(
            "range: { min: 0, max: 5 }\ndistribution: { type: normal, mean: 2, stddev: 1, exponent: 1 }",
            ScalarKind::Double,
        ),
        &format!("{ns}.distribution"),
        "unknown key \"exponent\" for type normal",
    );
    expect_err(
        parse_scalar(
            "range: { min: 0, max: 5 }\ndistribution: { type: exponential, mean: 2, stddev: 1 }",
            ScalarKind::Double,
        ),
        &format!("{ns}.distribution"),
        "unknown key \"stddev\" for type exponential",
    );
    expect_err(
        parse_scalar(
            "range: { min: 0, max: 5 }\ndistribution: { type: uniform, mean: 1 }",
            ScalarKind::Double,
        ),
        &format!("{ns}.distribution"),
        "unknown key \"mean\" for type uniform",
    );
}

// ===========================================================================
// Parsing: attribute types
// ===========================================================================

/// Scenario: Scalar annotations are parsed for each primitive type.
/// Guarantees: Every scalar attribute type yields `AttributeSpec::Scalar`.
#[test]
fn parse_attribute_scalars_for_every_primitive() {
    let cases = [
        (PrimitiveOrArrayTypeSpec::Int, "range: { min: 1, max: 9 }"),
        (
            PrimitiveOrArrayTypeSpec::Double,
            "range: { min: 0, max: 1 }",
        ),
        (PrimitiveOrArrayTypeSpec::String, "values: [a, b]"),
        (
            PrimitiveOrArrayTypeSpec::Boolean,
            "values: [true, false]\nweights: [1, 9]",
        ),
        (PrimitiveOrArrayTypeSpec::Any, r#"values: ["x", 1, true]"#),
    ];
    for (p, src) in cases {
        let spec = parse_attr(src, &primitive(p.clone())).unwrap_or_else(|e| panic!("{p:?}: {e}"));
        assert!(matches!(spec, AttributeSpec::Scalar(_)), "{p:?}");
    }
}

/// Scenario: Array annotations with `length` and `items` are parsed for every
/// array type.
/// Guarantees: Every array attribute type yields `AttributeSpec::Array`.
#[test]
fn parse_attribute_arrays_for_every_array_type() {
    let cases = [
        (
            PrimitiveOrArrayTypeSpec::Strings,
            "length: { range: { min: 1, max: 4 } }\nitems: { values: [a, b] }",
        ),
        (
            PrimitiveOrArrayTypeSpec::Ints,
            "length: { values: [3] }\nitems: { range: { min: 0, max: 255 } }",
        ),
        (
            PrimitiveOrArrayTypeSpec::Doubles,
            "length: { values: [1, 2], weights: [1, 1] }\nitems: { range: { min: 0, max: 1 }, distribution: { type: normal, mean: 0.5, stddev: 0.1 } }",
        ),
        (
            PrimitiveOrArrayTypeSpec::Booleans,
            "length: { range: { min: 0, max: 3 } }\nitems: { values: [true, false] }",
        ),
    ];
    for (p, src) in cases {
        let spec = parse_attr(src, &primitive(p.clone())).unwrap_or_else(|e| panic!("{p:?}: {e}"));
        assert!(matches!(spec, AttributeSpec::Array { .. }), "{p:?}");
    }
}

/// Scenario: Template annotations are parsed for every template type.
/// Guarantees: Each yields `AttributeSpec::Template` with a scalar or array
/// inner value matching the template's inner type.
#[test]
fn parse_attribute_templates_for_every_template_type() {
    let scalar_value = [
        (TemplateTypeSpec::String, "value: { values: [prod] }"),
        (
            TemplateTypeSpec::Int,
            "value: { range: { min: 0, max: 15 } }",
        ),
        (
            TemplateTypeSpec::Double,
            "value: { range: { min: 0, max: 1 } }",
        ),
        (TemplateTypeSpec::Boolean, "value: { values: [true] }"),
        (TemplateTypeSpec::Any, r#"value: { values: ["x", 1] }"#),
    ];
    for (t, v) in scalar_value {
        let spec = parse_attr(
            &format!("keys: {{ values: [env, team] }}\n{v}"),
            &AttributeType::Template(t.clone()),
        )
        .unwrap_or_else(|e| panic!("{t:?}: {e}"));
        match spec {
            AttributeSpec::Template { value, .. } => {
                assert!(matches!(*value, AttributeSpec::Scalar(_)), "{t:?}")
            }
            other => panic!("{t:?}: {other:?}"),
        }
    }
    let array_value = [
        (TemplateTypeSpec::Strings, "items: { values: [a] }"),
        (TemplateTypeSpec::Ints, "items: { values: [80, 443] }"),
        (
            TemplateTypeSpec::Doubles,
            "items: { range: { min: 0, max: 1 } }",
        ),
        (
            TemplateTypeSpec::Booleans,
            "items: { values: [true, false] }",
        ),
    ];
    for (t, items) in array_value {
        let spec = parse_attr(
            &format!("keys: {{ values: [k] }}\nvalue:\n  length: {{ values: [1] }}\n  {items}"),
            &AttributeType::Template(t.clone()),
        )
        .unwrap_or_else(|e| panic!("{t:?}: {e}"));
        match spec {
            AttributeSpec::Template { value, .. } => {
                assert!(matches!(*value, AttributeSpec::Array { .. }), "{t:?}")
            }
            other => panic!("{t:?}: {other:?}"),
        }
    }
}

/// Scenario: Keys are used in the wrong attribute context.
/// Guarantees: Scalar, array, and template annotations reject keys that only
/// belong to another context, naming the offending key path.
#[test]
fn parse_attribute_rejects_wrong_context_keys() {
    let ns = annotation::NAMESPACE;
    expect_err(
        parse_attr(
            "values: [1]\nitems: { values: [1] }",
            &primitive(PrimitiveOrArrayTypeSpec::Int),
        ),
        ns,
        "only valid on array attributes",
    );
    expect_err(
        parse_attr(
            "values: [a]\nkeys: { values: [a] }",
            &primitive(PrimitiveOrArrayTypeSpec::String),
        ),
        ns,
        "only valid on template attributes",
    );
    expect_err(
        parse_attr("values: [1, 2]", &primitive(PrimitiveOrArrayTypeSpec::Ints)),
        ns,
        "require `length` and `items`",
    );
    expect_err(
        parse_attr(
            "length: { values: [1] }",
            &primitive(PrimitiveOrArrayTypeSpec::Ints),
        ),
        ns,
        "missing required `items`",
    );
    expect_err(
        parse_attr(
            "length: { range: { min: -1, max: 2 } }\nitems: { values: [1] }",
            &primitive(PrimitiveOrArrayTypeSpec::Ints),
        ),
        &format!("{ns}.length.range.min"),
        "must be in 0..=32",
    );
    expect_err(
        parse_attr(
            "length: { values: [1] }\nitems: { values: [1] }\nextra: 1",
            &primitive(PrimitiveOrArrayTypeSpec::Ints),
        ),
        ns,
        "unknown key \"extra\"",
    );
    expect_err(
        parse_attr(
            "length: { values: [1] }\nitems: { values: [x] }",
            &primitive(PrimitiveOrArrayTypeSpec::Ints),
        ),
        &format!("{ns}.items.values[0]"),
        "does not match attribute type int",
    );
    expect_err(
        parse_attr(
            "values: [x]",
            &AttributeType::Template(TemplateTypeSpec::String),
        ),
        ns,
        "unknown key \"values\"",
    );
    expect_err(
        parse_attr(
            "keys: { values: [k] }",
            &AttributeType::Template(TemplateTypeSpec::String),
        ),
        ns,
        "missing required `value`",
    );
    expect_err(
        parse_attr(
            "keys: { values: [1] }\nvalue: { values: [x] }",
            &AttributeType::Template(TemplateTypeSpec::String),
        ),
        &format!("{ns}.keys.values[0]"),
        "does not match attribute type string",
    );
    expect_err(
        parse_attr(
            "keys: { values: [k] }\nvalue: { values: [x] }",
            &AttributeType::Template(TemplateTypeSpec::Ints),
        ),
        &format!("{ns}.value"),
        "require `length` and `items`",
    );
}

/// Scenario: Enum annotations list member values, non-members, or a range.
/// Guarantees: Subsets of members are accepted; non-members and the
/// continuous form are rejected with the offending index.
#[test]
fn parse_enum_membership() {
    let ty = enum_type(&["GET", "POST", "DELETE"]);
    let spec = parse_attr("values: [GET, POST]\nweights: [8, 2]", &ty).expect("subset is valid");
    assert!(matches!(
        spec,
        AttributeSpec::Scalar(ValueSpec::Discrete { .. })
    ));
    expect_err(
        parse_attr("values: [GET, PATCH]", &ty),
        &format!("{}.values[1]", annotation::NAMESPACE),
        "not an enum member",
    );
    let _ = parse_attr("range: { min: 0, max: 1 }", &ty).expect_err("range rejected for enum");
}

/// Scenario: Int-valued enum members are annotated.
/// Guarantees: Int member values are matched against int annotation values.
#[test]
fn parse_int_enum_membership() {
    let ty = AttributeType::Enum {
        members: [1i64, 2]
            .iter()
            .map(|v| EnumEntriesSpec {
                id: v.to_string(),
                value: SemconvValueSpec::Int(*v),
                brief: None,
                note: None,
                stability: None,
                deprecated: None,
                annotations: None,
            })
            .collect(),
    };
    let _ = parse_attr("values: [1, 2]\nweights: [9, 1]", &ty).expect("valid");
    expect_err(
        parse_attr("values: [3]", &ty),
        &format!("{}.values[0]", annotation::NAMESPACE),
        "not an enum member",
    );
}

// ===========================================================================
// Parsing: `weights` over attribute `examples`
// ===========================================================================

/// Parse a YAML list fixture into registry `Examples`.
fn examples(src: &str) -> Examples {
    serde_json::from_value(yaml_from_str(src)).expect("valid examples")
}

/// Scenario: Scalar attributes of every primitive kind carry `weights`
/// without `values` and have list or single `examples`.
/// Guarantees: The discrete form is selected with the examples, in order, as
/// `values` and the annotation weights unchanged; a single example becomes a
/// one-element list.
#[test]
fn parse_weights_use_examples_as_values() {
    let cases = [
        (
            PrimitiveOrArrayTypeSpec::String,
            Examples::Strings(vec!["a".into(), "b".into()]),
            "weights: [3, 1]",
            vec![Scalar::String("a".into()), Scalar::String("b".into())],
            vec![3.0, 1.0],
        ),
        (
            PrimitiveOrArrayTypeSpec::Int,
            Examples::Ints(vec![7, 9, 11]),
            "weights: [1, 2, 3]",
            vec![Scalar::Int(7), Scalar::Int(9), Scalar::Int(11)],
            vec![1.0, 2.0, 3.0],
        ),
        (
            PrimitiveOrArrayTypeSpec::Double,
            examples("[1.5, 2]"),
            "weights: [1, 1]",
            vec![Scalar::Double(1.5), Scalar::Double(2.0)],
            vec![1.0, 1.0],
        ),
        (
            PrimitiveOrArrayTypeSpec::Boolean,
            Examples::Bools(vec![true, false]),
            "weights: [9, 1]",
            vec![Scalar::Bool(true), Scalar::Bool(false)],
            vec![9.0, 1.0],
        ),
        (
            PrimitiveOrArrayTypeSpec::Any,
            examples(r##"["#ff0000", true, 3]"##),
            "weights: [4, 3, 2]",
            vec![
                Scalar::String("#ff0000".into()),
                Scalar::Bool(true),
                Scalar::Int(3),
            ],
            vec![4.0, 3.0, 2.0],
        ),
        (
            PrimitiveOrArrayTypeSpec::Int,
            Examples::Int(42),
            "weights: [1]",
            vec![Scalar::Int(42)],
            vec![1.0],
        ),
    ];
    for (p, ex, src, values, weights) in cases {
        let spec = parse_attr_with_examples(src, &primitive(p.clone()), &ex)
            .unwrap_or_else(|e| panic!("{p:?}: {e}"));
        assert_eq!(
            spec,
            AttributeSpec::Scalar(ValueSpec::Discrete {
                values,
                weights: Some(weights),
            }),
            "{p:?}"
        );
    }
}

/// Scenario: An enum attribute carries `weights` without `values` and its
/// `examples` are member values, or include a non-member.
/// Guarantees: Member examples are accepted as `values`; a non-member example
/// is rejected at its index and the message says the values came from the
/// attribute examples.
#[test]
fn parse_enum_weights_use_examples_as_values() {
    let ty = enum_type(&["heap", "non_heap"]);
    let spec = parse_attr_with_examples(
        "weights: [4, 1]",
        &ty,
        &Examples::Strings(vec!["heap".into(), "non_heap".into()]),
    )
    .expect("member examples are valid");
    assert_eq!(
        spec,
        AttributeSpec::Scalar(ValueSpec::Discrete {
            values: vec![
                Scalar::String("heap".into()),
                Scalar::String("non_heap".into())
            ],
            weights: Some(vec![4.0, 1.0]),
        })
    );
    let err = parse_attr_with_examples(
        "weights: [4, 1]",
        &ty,
        &Examples::Strings(vec!["heap".into(), "stack".into()]),
    )
    .expect_err("non-member example");
    assert_eq!(err.path, format!("{}.values[1]", annotation::NAMESPACE));
    assert!(err.message.contains("not an enum member"), "{err}");
    assert!(err.message.contains("attribute `examples`"), "{err}");
}

/// Scenario: Weights-only annotations are parsed where examples are missing,
/// mismatched, or not applicable.
/// Guarantees: Missing examples, a weights length that differs from the
/// examples length, and examples that do not match the attribute type are
/// rejected with the key path (noting the examples origin); explicit
/// `values` take precedence over examples; array `items` and template `keys`
/// still require `values`.
#[test]
fn parse_weights_with_examples_rejections() {
    let ns = annotation::NAMESPACE;
    let string = primitive(PrimitiveOrArrayTypeSpec::String);
    expect_err(
        parse_attr("weights: [1, 1]", &string),
        ns,
        "`weights` without `values` requires attribute `examples`",
    );
    expect_err(
        parse_attr("weights: [1]", &enum_type(&["a"])),
        ns,
        "`weights` without `values` requires attribute `examples`",
    );

    let ab = Examples::Strings(vec!["a".into(), "b".into()]);
    let err = parse_attr_with_examples("weights: [1, 2, 3]", &string, &ab)
        .expect_err("weights length mismatch");
    assert_eq!(err.path, format!("{ns}.weights"));
    assert!(
        err.message.contains("does not match `values` length 2"),
        "{err}"
    );
    assert!(err.message.contains("attribute `examples`"), "{err}");

    let err = parse_attr_with_examples(
        "weights: [1, 1]",
        &primitive(PrimitiveOrArrayTypeSpec::Int),
        &ab,
    )
    .expect_err("string examples on an int attribute");
    assert_eq!(err.path, format!("{ns}.values[0]"));
    assert!(
        err.message.contains("does not match attribute type int"),
        "{err}"
    );

    let spec = parse_attr_with_examples("values: [x]\nweights: [1]", &string, &ab)
        .expect("explicit values win");
    assert_eq!(
        spec,
        AttributeSpec::Scalar(ValueSpec::Discrete {
            values: vec![Scalar::String("x".into())],
            weights: Some(vec![1.0]),
        })
    );
    // `range` keeps its meaning: examples are not injected next to it.
    expect_err(
        parse_attr_with_examples(
            "range: { min: 0, max: 1 }\nweights: [1]",
            &primitive(PrimitiveOrArrayTypeSpec::Int),
            &Examples::Ints(vec![0]),
        ),
        ns,
        "`weights` requires `values`",
    );

    expect_err(
        parse_attr_with_examples(
            "length: { values: [1] }\nitems: { weights: [1, 1] }",
            &primitive(PrimitiveOrArrayTypeSpec::Strings),
            &Examples::Strings(vec!["a".into(), "b".into()]),
        ),
        &format!("{ns}.items"),
        "a value spec requires `values` or `range`",
    );
    expect_err(
        parse_attr_with_examples(
            "keys: { weights: [1] }\nvalue: { values: [v] }",
            &AttributeType::Template(TemplateTypeSpec::String),
            &ab,
        ),
        &format!("{ns}.keys"),
        "a value spec requires `values` or `range`",
    );
}

/// Scenario: The `traffic-generator` namespace is absent, null, or not a map.
/// Guarantees: Absent and null mean "no annotation"; a non-map is an error.
#[test]
fn lookup_namespace_shapes() {
    assert!(annotation::lookup(None).expect("ok").is_none());
    let mut m = BTreeMap::new();
    let _ = m.insert("code_generation".to_owned(), YamlValue(Value::Null));
    assert!(annotation::lookup(Some(&m)).expect("ok").is_none());
    let _ = m.insert(annotation::NAMESPACE.to_owned(), YamlValue(Value::Null));
    assert!(annotation::lookup(Some(&m)).expect("ok").is_none());
    let _ = m.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(Value::String("x".into())),
    );
    assert!(annotation::lookup(Some(&m)).is_err());
}

// ===========================================================================
// Parsing: signal annotations
// ===========================================================================

/// Scenario: A full histogram annotation is parsed.
/// Guarantees: All three histogram keys are captured, including bounds.
#[test]
fn parse_histogram_metric_spec() {
    let spec = annotation::parse_metric_spec(
        &yaml_map(
            "value: { range: { min: 0.001, max: 10 }, distribution: { type: lognormal, mean: 0.2, stddev: 0.5 } }\n\
             observations: { range: { min: 10, max: 50 } }\n\
             bounds: [0.005, 0.01, 0.1, 1]",
        ),
        &InstrumentSpec::Histogram,
    )
    .expect("valid");
    assert!(spec.value.is_some());
    assert!(spec.observations.is_some());
    assert_eq!(spec.bounds, Some(vec![0.005, 0.01, 0.1, 1.0]));
}

/// Scenario: Invalid metric annotations are parsed.
/// Guarantees: Histogram-only keys on other instruments, negative counter
/// increments and histogram observations, non-increasing bounds, zero
/// observations, and the unsupported `data_points` key (metrics carry one
/// data point) are rejected.
#[test]
fn parse_metric_spec_rejections() {
    let ns = annotation::NAMESPACE;
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { values: [10] }"),
            &InstrumentSpec::Gauge,
        ),
        ns,
        "only valid on histograms",
    );
    expect_err(
        annotation::parse_metric_spec(&yaml_map("bounds: [1, 2]"), &InstrumentSpec::Counter),
        ns,
        "only valid on histograms",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("value: { range: { min: -5, max: 5 } }"),
            &InstrumentSpec::Counter,
        ),
        &format!("{ns}.value.range.min"),
        ">= 0",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("value: { values: [1.5, -0.5] }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.value.values[1]"),
        ">= 0",
    );
    let _ = annotation::parse_metric_spec(
        &yaml_map("value: { range: { min: -5, max: 5 } }"),
        &InstrumentSpec::UpDownCounter,
    )
    .expect("negative updowncounter values are valid");
    expect_err(
        annotation::parse_metric_spec(&yaml_map("bounds: [1, 0.5]"), &InstrumentSpec::Histogram),
        &format!("{ns}.bounds[1]"),
        "strictly increasing",
    );
    expect_err(
        annotation::parse_metric_spec(&yaml_map("bounds: []"), &InstrumentSpec::Histogram),
        &format!("{ns}.bounds"),
        "must not be empty",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("data_points: { values: [2] }"),
            &InstrumentSpec::Gauge,
        ),
        ns,
        "unknown key \"data_points\" for metric group",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { range: { min: 0, max: 3 } }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.observations.range.min"),
        "must be in 1..=50",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("severity: { values: [INFO] }"),
            &InstrumentSpec::Gauge,
        ),
        ns,
        "unknown key \"severity\" for metric group",
    );
}

/// Scenario: `temporality` is annotated on every instrument with `delta`,
/// `cumulative`, an unknown name, and a non-string value.
/// Guarantees: `delta` is accepted on counters, updowncounters, and
/// histograms and is absent (generation falls back to `Unspecified`) when not
/// annotated; it is rejected on gauges, and `cumulative` (which a stateless
/// generator cannot honor), unknown names, and non-strings are rejected with
/// the `temporality` key path.
#[test]
fn parse_metric_temporality() {
    let ns = annotation::NAMESPACE;
    for instrument in [
        InstrumentSpec::Counter,
        InstrumentSpec::UpDownCounter,
        InstrumentSpec::Histogram,
    ] {
        let spec = annotation::parse_metric_spec(&yaml_map("temporality: delta"), &instrument)
            .expect("delta is valid on sums and histograms");
        assert_eq!(spec.temporality, Some(AggregationTemporality::Delta));
        let spec = annotation::parse_metric_spec(&yaml_map("{}"), &instrument).expect("valid");
        assert_eq!(spec.temporality, None);
    }
    expect_err(
        annotation::parse_metric_spec(&yaml_map("temporality: delta"), &InstrumentSpec::Gauge),
        ns,
        "only valid on sums and histograms",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("temporality: cumulative"),
            &InstrumentSpec::Counter,
        ),
        &format!("{ns}.temporality"),
        "unsupported temporality \"cumulative\" (expected delta)",
    );
    expect_err(
        annotation::parse_metric_spec(&yaml_map("temporality: Delta"), &InstrumentSpec::Counter),
        &format!("{ns}.temporality"),
        "unsupported temporality",
    );
    expect_err(
        annotation::parse_metric_spec(&yaml_map("temporality: 1"), &InstrumentSpec::Histogram),
        &format!("{ns}.temporality"),
        "must be a string",
    );
}

/// Scenario: Span and event annotations use valid and invalid keys/values.
/// Guarantees: Valid status/severity names parse; unknown names, negative
/// durations, and keys from other signal types are rejected.
#[test]
fn parse_span_and_event_specs() {
    let ns = annotation::NAMESPACE;
    let span = annotation::parse_span_spec(&yaml_map(
        "duration: { range: { min: 1, max: 5000 }, distribution: { type: exponential, mean: 120 } }\n\
         status: { values: [unset, ok, error], weights: [80, 15, 5] }",
    ))
    .expect("valid");
    assert!(span.duration.is_some() && span.status.is_some());
    expect_err(
        annotation::parse_span_spec(&yaml_map("status: { values: [failed] }")),
        &format!("{ns}.status.values[0]"),
        "unknown status \"failed\"",
    );
    expect_err(
        annotation::parse_span_spec(&yaml_map("duration: { range: { min: -1, max: 5 } }")),
        &format!("{ns}.duration.range.min"),
        ">= 0",
    );
    expect_err(
        annotation::parse_span_spec(&yaml_map("severity: { values: [INFO] }")),
        ns,
        "unknown key \"severity\" for span group",
    );

    let event = annotation::parse_event_spec(&yaml_map(
        "severity: { values: [INFO, WARN2, ERROR, FATAL4], weights: [90, 5, 4, 1] }",
    ))
    .expect("valid");
    assert!(event.severity.is_some());
    expect_err(
        annotation::parse_event_spec(&yaml_map("severity: { values: [LOUD] }")),
        &format!("{ns}.severity.values[0]"),
        "unknown severity \"LOUD\"",
    );
    expect_err(
        annotation::parse_event_spec(&yaml_map("duration: { values: [1] }")),
        ns,
        "unknown key \"duration\" for event group",
    );
}

/// Scenario: Every status and severity name in the mapping tables is looked
/// up, along with unknown names.
/// Guarantees: Each name maps to the value of its `StatusCode` /
/// `SeverityNumber` proto enum (spot-checked against the enums directly), and
/// unknown names map to `None`, so parse-time validation and generation agree.
#[test]
fn severity_and_status_mappings() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::SeverityNumber;
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::status::StatusCode;

    for (name, sev) in annotation::SEVERITY_TABLE {
        assert_eq!(annotation::severity(name), Some((sev as i32, name)));
    }
    assert_eq!(
        annotation::severity("INFO"),
        Some((SeverityNumber::Info as i32, "INFO"))
    );
    assert_eq!(
        annotation::severity("FATAL4"),
        Some((SeverityNumber::Fatal4 as i32, "FATAL4"))
    );
    assert_eq!(annotation::severity("LOUD"), None);

    assert_eq!(
        annotation::status_code("unset"),
        Some(StatusCode::Unset as i32)
    );
    assert_eq!(annotation::status_code("ok"), Some(StatusCode::Ok as i32));
    assert_eq!(
        annotation::status_code("error"),
        Some(StatusCode::Error as i32)
    );
    assert_eq!(annotation::status_code("bogus"), None);
}
