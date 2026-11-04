// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for `traffic-generator` annotation parsing, sampling, registry
//! validation, and signal generation against the vendored semantic
//! conventions registry.
//!
//! All randomized tests use a seeded `StdRng` so they are deterministic and
//! independent of execution order.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value as AnyValueKind;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, metric::Data, number_data_point,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde_yaml::{Mapping, Value};
use weaver_forge::v1::registry::ResolvedRegistry;
use weaver_resolved_schema::v1::attribute::Attribute;
use weaver_semconv::YamlValue;
use weaver_semconv::v1::attribute::{
    AttributeType, EnumEntriesSpec, Examples, PrimitiveOrArrayTypeSpec, TemplateTypeSpec,
    ValueSpec as SemconvValueSpec,
};
use weaver_semconv::v1::group::{GroupType, InstrumentSpec};

use super::annotation::{
    self, AnnotationError, AttributeSpec, DEFAULT_HISTOGRAM_BOUNDS, Distribution, Range, Scalar,
    ScalarKind, ValueSpec,
};
use super::attributes::{AttributeGen, get_attribute_name_value};
use super::config::{Config, TrafficConfig, default_registry_path};
use super::sampler::{
    IndexSampler, NumberSampler, ValueSampler, lognormal_params, normal_cdf, normal_inverse_sf,
    normal_quantile, normal_sf, sample_count, sample_value,
};
use super::semconv_model::SemconvModel;
use super::semconv_signal::{
    bucketize, semconv_otlp_logs_with_rng, semconv_otlp_metrics_with_rng,
    semconv_otlp_traces_with_rng, severity_number, status_code,
};

// ===========================================================================
// Helpers
// ===========================================================================

const SEED: u64 = 0x5eed_cafe;

fn rng() -> StdRng {
    StdRng::seed_from_u64(SEED)
}

fn yaml_map(src: &str) -> Mapping {
    match serde_yaml::from_str::<Value>(src).expect("valid yaml") {
        Value::Mapping(m) => m,
        other => panic!("expected mapping, got {other:?}"),
    }
}

fn parse_scalar(src: &str, kind: ScalarKind) -> Result<ValueSpec, AnnotationError> {
    annotation::parse_value_spec(
        &yaml_map(src),
        annotation::NAMESPACE,
        kind,
        &["values", "weights", "range", "distribution"],
    )
}

fn primitive(p: PrimitiveOrArrayTypeSpec) -> AttributeType {
    AttributeType::PrimitiveOrArray(p)
}

fn parse_attr(src: &str, ty: &AttributeType) -> Result<AttributeSpec, AnnotationError> {
    annotation::parse_attribute_spec(&yaml_map(src), ty)
}

fn expect_err(result: Result<impl std::fmt::Debug, AnnotationError>, path: &str, needle: &str) {
    match result {
        Ok(v) => panic!("expected error at {path} containing {needle:?}, got Ok({v:?})"),
        Err(e) => {
            assert_eq!(e.path, path, "unexpected error path for {e}");
            assert!(
                e.message.contains(needle),
                "error message {:?} does not contain {needle:?}",
                e.message
            );
        }
    }
}

fn enum_type(values: &[&str]) -> AttributeType {
    AttributeType::Enum {
        members: values
            .iter()
            .map(|v| EnumEntriesSpec {
                id: (*v).to_owned(),
                value: SemconvValueSpec::String((*v).to_owned()),
                brief: None,
                note: None,
                stability: None,
                deprecated: None,
                annotations: None,
            })
            .collect(),
    }
}

fn attribute(name: &str, ty: AttributeType, annotation: Option<&str>) -> Attribute {
    Attribute {
        name: name.to_owned(),
        r#type: ty,
        brief: "brief".to_owned(),
        examples: None,
        tag: None,
        requirement_level: Default::default(),
        sampling_relevant: None,
        note: String::new(),
        stability: None,
        deprecated: None,
        prefix: false,
        tags: None,
        annotations: annotation.map(|src| {
            let mut m = BTreeMap::new();
            let _ = m.insert(
                annotation::NAMESPACE.to_owned(),
                YamlValue(serde_yaml::from_str(src).expect("valid yaml")),
            );
            m
        }),
        value: None,
        role: None,
    }
}

fn number_sampler(range: (f64, f64), d: Distribution, integer: bool) -> NumberSampler {
    NumberSampler::new(
        Range {
            min: range.0,
            max: range.1,
        },
        d,
        integer,
    )
}

fn draw(sampler: &NumberSampler, n: usize) -> Vec<f64> {
    let mut r = rng();
    (0..n).map(|_| sampler.sample_f64(&mut r)).collect()
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn quantile(xs: &[f64], q: f64) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn any_scalar(v: &AnyValue) -> Scalar {
    match v.value.as_ref().expect("value set") {
        AnyValueKind::IntValue(i) => Scalar::Int(*i),
        AnyValueKind::DoubleValue(d) => Scalar::Double(*d),
        AnyValueKind::StringValue(s) => Scalar::String(s.clone()),
        AnyValueKind::BoolValue(b) => Scalar::Bool(*b),
        other => panic!("not a scalar: {other:?}"),
    }
}

fn kv_scalar(kv: &KeyValue) -> Scalar {
    any_scalar(kv.value.as_ref().expect("kv value"))
}

static REGISTRY: OnceLock<ResolvedRegistry> = OnceLock::new();
static MODEL: OnceLock<SemconvModel> = OnceLock::new();

fn model() -> &'static SemconvModel {
    MODEL.get_or_init(|| {
        SemconvModel::compile(registry())
            .unwrap_or_else(|errors| panic!("{}", annotation::format_errors(&errors)))
    })
}

fn registry() -> &'static ResolvedRegistry {
    REGISTRY.get_or_init(|| {
        let config = Config::new(
            TrafficConfig::new(None, None, 10, 1, 1, 1),
            default_registry_path(),
        );
        config
            .get_registry()
            .expect("vendored registry resolves")
            .expect("semantic conventions data source")
    })
}

fn groups_of(ty: GroupType) -> Vec<&'static weaver_forge::v1::registry::ResolvedGroup> {
    registry()
        .groups
        .iter()
        .filter(|g| g.r#type == ty)
        .collect()
}

/// Every non-uniform continuous distribution exercised by the range tests,
/// paired with ranges that put the bulk inside, in a tail, or straddling it.
fn non_uniform_cases() -> Vec<(&'static str, Distribution, (f64, f64))> {
    vec![
        (
            "normal-centered",
            Distribution::Normal {
                mean: 50.0,
                stddev: 10.0,
            },
            (0.0, 100.0),
        ),
        (
            "normal-wide",
            Distribution::Normal {
                mean: 8080.0,
                stddev: 20000.0,
            },
            (1024.0, 65535.0),
        ),
        (
            "normal-left-tail",
            Distribution::Normal {
                mean: 0.0,
                stddev: 1.0,
            },
            (-10.0, -8.0),
        ),
        (
            "normal-right-tail",
            Distribution::Normal {
                mean: 0.0,
                stddev: 1.0,
            },
            (8.0, 10.0),
        ),
        (
            "normal-mean-outside",
            Distribution::Normal {
                mean: 0.0,
                stddev: 2.0,
            },
            (2.0, 5.0),
        ),
        (
            "normal-negative",
            Distribution::Normal {
                mean: -40.0,
                stddev: 5.0,
            },
            (-60.0, -20.0),
        ),
        (
            "lognormal-latency",
            Distribution::LogNormal {
                mean: 0.15,
                stddev: 0.3,
            },
            (0.001, 30.0),
        ),
        (
            "lognormal-bytes",
            Distribution::LogNormal {
                mean: 2048.0,
                stddev: 8192.0,
            },
            (1.0, 1_048_576.0),
        ),
        (
            "lognormal-range-from-zero",
            Distribution::LogNormal {
                mean: 10.0,
                stddev: 5.0,
            },
            (0.0, 50.0),
        ),
        (
            "lognormal-right-tail",
            Distribution::LogNormal {
                mean: 1.0,
                stddev: 0.1,
            },
            (5.0, 6.0),
        ),
        (
            "exponential-small-mean",
            Distribution::Exponential { mean: 2.0 },
            (0.0, 50.0),
        ),
        (
            "exponential-truncated",
            Distribution::Exponential { mean: 10000.0 },
            (1.0, 5000.0),
        ),
        (
            "exponential-far-tail",
            Distribution::Exponential { mean: 1.0 },
            (100.0, 101.0),
        ),
        (
            "exponential-negative-min",
            Distribution::Exponential { mean: 5.0 },
            (-10.0, 20.0),
        ),
    ]
}

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
            format!("{ns}.range"),
            "integer bounds",
        ),
        (
            "range: { min: 0, max: .inf }",
            ScalarKind::Double,
            format!("{ns}.range.max"),
            "finite",
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
        ">= 0",
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
/// Guarantees: All four histogram keys are captured, including bounds.
#[test]
fn parse_histogram_metric_spec() {
    let spec = annotation::parse_metric_spec(
        &yaml_map(
            "value: { range: { min: 0.001, max: 10 }, distribution: { type: lognormal, mean: 0.2, stddev: 0.5 } }\n\
             observations: { range: { min: 10, max: 1000 } }\n\
             bounds: [0.005, 0.01, 0.1, 1]\n\
             data_points: { values: [1, 2] }",
        ),
        &InstrumentSpec::Histogram,
    )
    .expect("valid");
    assert!(spec.value.is_some());
    assert!(spec.observations.is_some());
    assert!(spec.data_points.is_some());
    assert_eq!(spec.bounds, Some(vec![0.005, 0.01, 0.1, 1.0]));
}

/// Scenario: Invalid metric annotations are parsed.
/// Guarantees: Histogram-only keys on other instruments, negative counter
/// increments, non-increasing bounds, and zero data points are rejected.
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
            &yaml_map("data_points: { values: [0] }"),
            &InstrumentSpec::Gauge,
        ),
        &format!("{ns}.data_points.values[0]"),
        ">= 1",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { range: { min: 0, max: 3 } }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.observations.range.min"),
        ">= 1",
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

/// Scenario: Severity and status names are mapped to OTLP numbers.
/// Guarantees: Every documented name maps to its protobuf value.
#[test]
fn severity_and_status_mappings() {
    for (i, name) in annotation::SEVERITY_NAMES.iter().enumerate() {
        assert_eq!(severity_number(name), Some(i as i32 + 1));
    }
    assert_eq!(severity_number("LOUD"), None);
    assert_eq!(status_code("unset"), Some(0));
    assert_eq!(status_code("ok"), Some(1));
    assert_eq!(status_code("error"), Some(2));
}

// ===========================================================================
// Sampling: standard normal functions
// ===========================================================================

/// Standard normal CDF reference values `(z, Phi(z))`.
///
/// Computed with 40-digit arithmetic (`mpmath`: `erfc(-z / sqrt(2)) / 2`) and
/// rounded to f64; they match published standard normal tables.
const CDF_REFERENCE: [(f64, f64); 15] = [
    (0.0, 0.5),
    (0.5, 0.691_462_461_274_013_1),
    (-0.5, 0.308_537_538_725_986_9),
    (1.0, 0.841_344_746_068_542_9),
    (-1.0, 0.158_655_253_931_457_05),
    (1.96, 0.975_002_104_851_779_6),
    (-1.96, 0.024_997_895_148_220_436),
    (3.0, 0.998_650_101_968_369_9),
    (-3.0, 0.001_349_898_031_630_094_5),
    (5.0, 0.999_999_713_348_428_1),
    (-5.0, 2.866_515_718_791_939e-7),
    (-8.0, 6.220_960_574_271_784e-16),
    (-10.0, 7.619_853_024_160_526e-24),
    (-20.0, 2.753_624_118_606_233_7e-89),
    (-37.0, 5.725_571_222_524_577e-300),
];

/// Standard normal quantile reference values `(p, z)` with `Phi(z) = p`.
///
/// Computed with 60-digit arithmetic (`mpmath`: root of
/// `log(Phi(z)) - log(p)`).
const QUANTILE_REFERENCE: [(f64, f64); 8] = [
    (0.975, 1.959_963_984_540_054_2),
    (0.995, 2.575_829_303_548_901),
    (0.001, -3.090_232_306_167_813_5),
    (1e-10, -6.361_340_902_404_056),
    (1e-20, -9.262_340_089_798_408),
    (1e-100, -21.273_453_560_965_324),
    (1e-300, -37.047_096_299_361_2),
    (0.5, 0.0),
];

fn rel_err(a: f64, b: f64) -> f64 {
    if a == b {
        0.0
    } else {
        (a - b).abs() / a.abs().max(b.abs()).max(f64::MIN_POSITIVE)
    }
}

fn z_grid() -> Vec<f64> {
    (-370..=370).map(|i| f64::from(i) * 0.1).collect()
}

fn p_grid() -> Vec<f64> {
    let mut ps: Vec<f64> = (1..=300).map(|e| 10f64.powi(-e)).collect();
    ps.extend((1..1000).map(|i| f64::from(i) / 1000.0));
    ps.extend((1..=16).map(|e| 1.0 - 10f64.powi(-e)));
    ps.sort_by(f64::total_cmp);
    ps
}

/// Scenario: `normal_cdf` and `normal_sf` are evaluated at tabulated points
/// from the center out to z = -37 (p ~ 1e-300).
/// Guarantees: Both match high-precision reference values to 1e-12 relative
/// error, and `normal_sf` keeps that precision in the upper tail where
/// `1 - cdf` would cancel to zero.
#[test]
fn normal_cdf_and_sf_match_reference_values() {
    for (z, expected) in CDF_REFERENCE {
        let cdf = normal_cdf(z);
        assert!(
            rel_err(cdf, expected) < 1e-12,
            "cdf({z}) = {cdf}, expected {expected}"
        );
        let sf = normal_sf(-z);
        assert!(
            rel_err(sf, expected) < 1e-12,
            "sf({}) = {sf}, expected {expected}",
            -z
        );
    }
}

/// Scenario: `normal_quantile` and `normal_inverse_sf` are evaluated at
/// tabulated probabilities from 0.5 down to 1e-300.
/// Guarantees: Both match high-precision reference quantiles to 1e-12
/// relative error (or 1e-15 absolute at zero), so truncated sampling places
/// values where the exact distribution would.
#[test]
fn normal_quantile_matches_reference_values() {
    for (p, expected) in QUANTILE_REFERENCE {
        let z = normal_quantile(p);
        assert!(
            rel_err(z, expected) < 1e-12 || (z - expected).abs() < 1e-15,
            "quantile({p}) = {z}, expected {expected}"
        );
        let upper = normal_inverse_sf(p);
        assert!(
            rel_err(upper, -expected) < 1e-12 || (upper + expected).abs() < 1e-15,
            "inverse_sf({p}) = {upper}, expected {}",
            -expected
        );
    }
}

/// Scenario: Symmetry identities of the standard normal are checked over a
/// dense grid of z in [-37, 37] and p from 1e-300 to 1 - 1e-16.
/// Guarantees: `cdf(z) + cdf(-z) = 1`, `sf(z) == cdf(-z)` exactly, and
/// `quantile(p) == -inverse_sf(p)` exactly, so lower- and upper-tail
/// sampling paths are consistent.
#[test]
fn normal_functions_are_symmetric() {
    for z in z_grid() {
        assert!(
            (normal_cdf(z) + normal_cdf(-z) - 1.0).abs() < 1e-15,
            "z = {z}"
        );
        assert_eq!(normal_sf(z), normal_cdf(-z), "z = {z}");
    }
    for p in p_grid() {
        assert_eq!(normal_quantile(p), -normal_inverse_sf(p), "p = {p}");
    }
}

/// Scenario: The CDF, survival function, and quantile are evaluated over
/// increasing inputs.
/// Guarantees: CDF and quantile are non-decreasing, the survival function is
/// non-increasing, and every probability lies in [0, 1], so the inverse
/// transform maps ordered uniforms to ordered samples.
#[test]
fn normal_functions_are_monotonic_and_bounded() {
    let zs = z_grid();
    for w in zs.windows(2) {
        assert!(normal_cdf(w[0]) <= normal_cdf(w[1]), "cdf at {}", w[1]);
        assert!(normal_sf(w[0]) >= normal_sf(w[1]), "sf at {}", w[1]);
    }
    for z in zs {
        assert!((0.0..=1.0).contains(&normal_cdf(z)));
        assert!((0.0..=1.0).contains(&normal_sf(z)));
    }
    let ps = p_grid();
    for w in ps.windows(2) {
        assert!(
            normal_quantile(w[0]) <= normal_quantile(w[1]),
            "quantile at {}",
            w[1]
        );
    }
}

/// Scenario: z is round-tripped through `normal_cdf` / `normal_quantile` (z <=
/// 0) and `normal_sf` / `normal_inverse_sf` (z > 0) over [-37, 37].
/// Guarantees: The quantile inverts the CDF to 1e-9 absolute error, which
/// verifies the Halley refinement on top of the rational approximation.
#[test]
fn normal_quantile_inverts_cdf() {
    for z in z_grid() {
        let back = if z <= 0.0 {
            normal_quantile(normal_cdf(z))
        } else {
            normal_inverse_sf(normal_sf(z))
        };
        assert!((back - z).abs() < 1e-9, "round trip of {z} gave {back}");
    }
}

/// Scenario: The derivative of the lower-tail probability (`normal_cdf` for
/// z <= 0, `normal_sf` for z > 0) is estimated by a central difference over z
/// in [-8, 8].
/// Guarantees: It matches the closed-form normal density
/// `exp(-z^2 / 2) / sqrt(2 pi)` to 1e-6 relative error, checking the CDF and
/// survival function against an independent formula.
#[test]
fn normal_cdf_derivative_matches_density() {
    for i in -80..=80 {
        let z = f64::from(i) * 0.1;
        // Differentiate the small tail probability so the difference does not
        // cancel against values near 1; scale the step with z.
        let h = 1e-6 * z.abs().max(1.0);
        let numeric = if z <= 0.0 {
            (normal_cdf(z + h) - normal_cdf(z - h)) / (2.0 * h)
        } else {
            (normal_sf(z - h) - normal_sf(z + h)) / (2.0 * h)
        };
        let density = (-z * z / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
        assert!(
            rel_err(numeric, density) < 1e-6,
            "z = {z}: d/dz cdf = {numeric}, density = {density}"
        );
    }
}

/// Scenario: The normal functions are evaluated at their domain edges.
/// Guarantees: Probabilities 0 and 1 map to infinite quantiles, infinite z
/// maps to exactly 0 or 1, and no probability in [0, 1] yields NaN.
#[test]
fn normal_functions_handle_edges() {
    assert_eq!(normal_quantile(0.0), f64::NEG_INFINITY);
    assert_eq!(normal_quantile(1.0), f64::INFINITY);
    assert_eq!(normal_inverse_sf(0.0), f64::INFINITY);
    assert_eq!(normal_inverse_sf(1.0), f64::NEG_INFINITY);
    assert_eq!(normal_cdf(f64::NEG_INFINITY), 0.0);
    assert_eq!(normal_cdf(f64::INFINITY), 1.0);
    assert_eq!(normal_sf(f64::NEG_INFINITY), 1.0);
    assert_eq!(normal_sf(f64::INFINITY), 0.0);
    for p in std::iter::once(0.0)
        .chain(p_grid())
        .chain(std::iter::once(1.0))
    {
        assert!(!normal_quantile(p).is_nan(), "quantile({p}) is NaN");
        assert!(!normal_inverse_sf(p).is_nan(), "inverse_sf({p}) is NaN");
    }
}

/// Scenario: Lognormal value moments are converted to the underlying normal
/// parameters for several (mean, stddev) pairs.
/// Guarantees: The closed-form lognormal moments of the returned (mu, sigma)
/// reproduce the requested mean and stddev, so annotations in value units
/// produce the intended distribution.
#[test]
fn lognormal_params_reproduce_moments() {
    for (mean_v, sd_v) in [
        (1.0, 1.0),
        (0.15, 0.3),
        (2048.0, 8192.0),
        (100.0, 1.0),
        (1e-3, 5e-3),
    ] {
        let (mu, sigma) = lognormal_params(mean_v, sd_v);
        let s2 = sigma * sigma;
        let mean = (mu + s2 / 2.0).exp();
        let sd = ((s2.exp() - 1.0) * (2.0 * mu + s2).exp()).sqrt();
        assert!(
            rel_err(mean, mean_v) < 1e-12,
            "mean for ({mean_v}, {sd_v}): {mean}"
        );
        assert!(
            rel_err(sd, sd_v) < 1e-10,
            "stddev for ({mean_v}, {sd_v}): {sd}"
        );
    }
}

// ===========================================================================
// Sampling: range guarantees for every non-uniform method
// ===========================================================================

/// Scenario: Every non-uniform continuous distribution is sampled as doubles
/// over ranges that are centered, wide, in either tail, negative, or with the
/// mean outside the range.
/// Guarantees: All samples are finite and lie inside [min, max].
#[test]
fn continuous_samples_stay_in_range_doubles() {
    for (name, d, (lo, hi)) in non_uniform_cases() {
        let xs = draw(&number_sampler((lo, hi), d, false), 20_000);
        for x in &xs {
            assert!(x.is_finite(), "{name}: non-finite {x}");
            assert!((lo..=hi).contains(x), "{name}: {x} outside [{lo}, {hi}]");
        }
    }
}

/// Scenario: Every non-uniform continuous distribution is sampled as integers.
/// Guarantees: All samples are integral and lie inside [min, max].
#[test]
fn continuous_samples_stay_in_range_ints() {
    for (name, d, (lo, hi)) in non_uniform_cases() {
        let (lo, hi) = (lo.ceil(), hi.floor());
        let xs = draw(&number_sampler((lo, hi), d, true), 20_000);
        for x in &xs {
            assert_eq!(x.fract(), 0.0, "{name}: non-integer {x}");
            assert!((lo..=hi).contains(x), "{name}: {x} outside [{lo}, {hi}]");
        }
    }
}

/// Scenario: Zipf is sampled over several integer ranges and exponents, with
/// both the cumulative table and the rejection-inversion sampler.
/// Guarantees: Every sample is an integer in [min, max] and rank 1 (min) is
/// the most frequent value.
#[test]
fn zipf_samples_stay_in_range_and_favor_min() {
    for (lo, hi, s) in [
        (1.0, 10.0, 1.0),
        (1.0, 1000.0, 1.1),
        (100.0, 200.0, 2.0),
        (-5.0, 5.0, 0.5),
    ] {
        let range = Range { min: lo, max: hi };
        let table = NumberSampler::new(range, Distribution::Zipf { exponent: s }, true);
        let rejection = NumberSampler::new_zipf_rejection(range, s);
        for (name, sampler) in [("table", table), ("rejection", rejection)] {
            let xs = draw(&sampler, 20_000);
            let mut counts: HashMap<i64, usize> = HashMap::new();
            for x in &xs {
                assert_eq!(x.fract(), 0.0, "{name}: non-integer {x}");
                assert!((lo..=hi).contains(x), "{name}: {x} outside [{lo}, {hi}]");
                *counts.entry(*x as i64).or_default() += 1;
            }
            let (mode, _) = counts.iter().max_by_key(|(_, c)| **c).expect("non-empty");
            assert_eq!(*mode, lo as i64, "{name} zipf({s}) on [{lo}, {hi}]");
        }
    }
}

/// Scenario: The integer sampler's widened inverse transform is used on a
/// small range.
/// Guarantees: Both endpoints are reachable and every integer in the range
/// appears, so rounding does not starve the bounds.
#[test]
fn integer_endpoints_are_reachable() {
    for d in [
        Distribution::Uniform,
        Distribution::Normal {
            mean: 3.0,
            stddev: 1.5,
        },
        Distribution::LogNormal {
            mean: 3.0,
            stddev: 2.0,
        },
        Distribution::Exponential { mean: 2.0 },
        Distribution::Zipf { exponent: 0.5 },
    ] {
        let xs = draw(&number_sampler((1.0, 5.0), d, true), 20_000);
        let seen: HashSet<i64> = xs.iter().map(|x| *x as i64).collect();
        assert_eq!(seen, (1..=5).collect(), "{d:?}");
    }
}

/// Scenario: Integer sampling is applied to a range with fractional bounds
/// (as can happen for nested specs constructed in code).
/// Guarantees: Samples are snapped inside the integer sub-range.
#[test]
fn integer_sampler_snaps_fractional_bounds() {
    let xs = draw(
        &number_sampler(
            (0.5, 3.5),
            Distribution::Normal {
                mean: 2.0,
                stddev: 5.0,
            },
            true,
        ),
        5_000,
    );
    assert!(xs.iter().all(|x| (1.0..=3.0).contains(x)));
}

/// Scenario: A degenerate range (min == max) is sampled with every
/// distribution type.
/// Guarantees: The constant value is always returned.
#[test]
fn degenerate_range_is_constant() {
    for d in [
        Distribution::Uniform,
        Distribution::Normal {
            mean: 0.0,
            stddev: 1.0,
        },
        Distribution::LogNormal {
            mean: 1.0,
            stddev: 1.0,
        },
        Distribution::Exponential { mean: 1.0 },
        Distribution::Zipf { exponent: 1.0 },
    ] {
        for integer in [false, true] {
            let xs = draw(&number_sampler((7.0, 7.0), d, integer), 100);
            assert!(xs.iter().all(|x| *x == 7.0), "{d:?} integer={integer}");
        }
    }
}

/// Scenario: A normal distribution is truncated to a range far in each tail.
/// Guarantees: Inverse transform keeps samples in range and spread across it
/// (no collapse onto a bound), with the mass piled toward the side nearest
/// the mean.
#[test]
fn far_tail_normal_is_spread_not_clamped() {
    let d = Distribution::Normal {
        mean: 0.0,
        stddev: 1.0,
    };
    let right = draw(&number_sampler((8.0, 10.0), d, false), 20_000);
    assert!(right.iter().all(|x| (8.0..=10.0).contains(x)));
    assert!(
        quantile(&right, 0.5) < 8.2,
        "median should hug the near bound"
    );
    assert!(
        quantile(&right, 0.99) > 8.3,
        "upper tail should not collapse"
    );
    let distinct: HashSet<u64> = right.iter().map(|x| x.to_bits()).collect();
    assert!(
        distinct.len() > 19_000,
        "values must not pile up on a bound"
    );

    let left = draw(&number_sampler((-10.0, -8.0), d, false), 20_000);
    assert!(left.iter().all(|x| (-10.0..=-8.0).contains(x)));
    assert!(quantile(&left, 0.5) > -8.2);
}

// ===========================================================================
// Sampling: distribution shape
// ===========================================================================

/// Scenario: Uniform doubles are sampled over [10, 20].
/// Guarantees: Mean and quartiles match the uniform distribution.
#[test]
fn uniform_shape() {
    let xs = draw(
        &number_sampler((10.0, 20.0), Distribution::Uniform, false),
        50_000,
    );
    assert!((mean(&xs) - 15.0).abs() < 0.05);
    assert!((quantile(&xs, 0.25) - 12.5).abs() < 0.1);
    assert!((quantile(&xs, 0.75) - 17.5).abs() < 0.1);
}

/// Scenario: A normal distribution is sampled over a range that contains
/// essentially all of its mass.
/// Guarantees: Sample mean, stddev and median match the parameters.
#[test]
fn normal_shape_untruncated() {
    let xs = draw(
        &number_sampler(
            (0.0, 100.0),
            Distribution::Normal {
                mean: 50.0,
                stddev: 10.0,
            },
            false,
        ),
        50_000,
    );
    let m = mean(&xs);
    let sd = (xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64).sqrt();
    assert!((m - 50.0).abs() < 0.2, "mean {m}");
    assert!((sd - 10.0).abs() < 0.2, "stddev {sd}");
    assert!((quantile(&xs, 0.5) - 50.0).abs() < 0.3);
    assert!(
        (quantile(&xs, 0.8413) - 60.0).abs() < 0.4,
        "one sigma quantile"
    );
}

/// Scenario: A normal distribution is truncated to [mean, +inf-ish).
/// Guarantees: The sample mean equals the half-normal mean
/// (mean + stddev * sqrt(2/pi)), confirming truncation is exact.
#[test]
fn normal_shape_half_truncated() {
    let xs = draw(
        &number_sampler(
            (0.0, 100.0),
            Distribution::Normal {
                mean: 0.0,
                stddev: 10.0,
            },
            false,
        ),
        50_000,
    );
    let expected = 10.0 * (2.0 / std::f64::consts::PI).sqrt();
    assert!((mean(&xs) - expected).abs() < 0.1, "mean {}", mean(&xs));
}

/// Scenario: A lognormal distribution is sampled over a wide range.
/// Guarantees: Mean and median match the value-space parameters
/// (median = exp(mu)).
#[test]
fn lognormal_shape() {
    let (mean_v, sd_v) = (100.0, 50.0);
    let xs = draw(
        &number_sampler(
            (0.0, 1e7),
            Distribution::LogNormal {
                mean: mean_v,
                stddev: sd_v,
            },
            false,
        ),
        50_000,
    );
    let (mu, _) = lognormal_params(mean_v, sd_v);
    assert!(
        (mean(&xs) - mean_v).abs() / mean_v < 0.02,
        "mean {}",
        mean(&xs)
    );
    assert!((quantile(&xs, 0.5) - mu.exp()).abs() / mu.exp() < 0.02);
}

/// Scenario: An exponential distribution is sampled untruncated and
/// truncated.
/// Guarantees: The untruncated mean matches; the truncated distribution keeps
/// the memoryless shape (min + mean when shifted far into the tail).
#[test]
fn exponential_shape() {
    let xs = draw(
        &number_sampler((0.0, 1e6), Distribution::Exponential { mean: 5.0 }, false),
        50_000,
    );
    assert!((mean(&xs) - 5.0).abs() < 0.1, "mean {}", mean(&xs));
    assert!((quantile(&xs, 0.5) - 5.0 * 2f64.ln()).abs() < 0.1);

    let tail = draw(
        &number_sampler((100.0, 1e6), Distribution::Exponential { mean: 5.0 }, false),
        50_000,
    );
    assert!(
        (mean(&tail) - 105.0).abs() < 0.15,
        "tail mean {}",
        mean(&tail)
    );
}

/// Scenario: Zipf is sampled over [1, 10] with exponent 1.
/// Guarantees: Rank frequencies follow 1/k normalized by the harmonic number.
#[test]
fn zipf_shape() {
    let xs = draw(
        &number_sampler((1.0, 10.0), Distribution::Zipf { exponent: 1.0 }, true),
        100_000,
    );
    let h: f64 = (1..=10).map(|k| 1.0 / f64::from(k)).sum();
    for k in 1..=10 {
        let observed = xs.iter().filter(|x| **x == f64::from(k)).count() as f64 / xs.len() as f64;
        let expected = 1.0 / (f64::from(k) * h);
        assert!(
            (observed - expected).abs() < 0.01,
            "rank {k}: {observed} vs {expected}"
        );
    }
}

/// Scenario: Weighted and unweighted discrete specs are sampled.
/// Guarantees: Proportions match the weights (uniform when omitted) and
/// zero-weight values are never chosen.
#[test]
fn discrete_proportions() {
    let mut r = rng();
    let weighted = IndexSampler::new(4, Some(&[70.0, 20.0, 10.0, 0.0]));
    let mut counts = [0usize; 4];
    for _ in 0..100_000 {
        counts[weighted.sample(&mut r)] += 1;
    }
    for (i, w) in [0.7, 0.2, 0.1, 0.0].iter().enumerate() {
        let p = counts[i] as f64 / 100_000.0;
        assert!((p - w).abs() < 0.01, "index {i}: {p} vs {w}");
    }
    assert_eq!(counts[3], 0, "zero weight must never be selected");

    let uniform = IndexSampler::new(5, None);
    let mut counts = [0usize; 5];
    for _ in 0..100_000 {
        counts[uniform.sample(&mut r)] += 1;
    }
    for c in counts {
        assert!((c as f64 / 100_000.0 - 0.2).abs() < 0.01);
    }
}

/// Scenario: Two samplers are driven by identically seeded RNGs.
/// Guarantees: Sampling is reproducible for a given seed.
#[test]
fn sampling_is_reproducible_for_a_seed() {
    let s = number_sampler(
        (0.0, 10.0),
        Distribution::LogNormal {
            mean: 2.0,
            stddev: 1.0,
        },
        false,
    );
    assert_eq!(draw(&s, 1_000), draw(&s, 1_000));
}

/// Scenario: `sample_value` and `sample_count` are called on continuous and
/// discrete specs.
/// Guarantees: Continuous int specs yield `Scalar::Int`, doubles yield
/// `Scalar::Double`, discrete values are returned as written, and
/// `sample_count` never goes negative.
#[test]
fn sample_value_output_types() {
    let spec = ValueSpec::Continuous {
        range: Range { min: 0.0, max: 9.0 },
        distribution: Distribution::Uniform,
    };
    let mut r = rng();
    assert!(matches!(sample_value(&spec, true, &mut r), Scalar::Int(_)));
    assert!(matches!(
        sample_value(&spec, false, &mut r),
        Scalar::Double(_)
    ));
    let neg = ValueSpec::Discrete {
        values: vec![Scalar::Int(-3)],
        weights: None,
    };
    assert_eq!(sample_value(&neg, true, &mut r), Scalar::Int(-3));
    assert_eq!(sample_count(&neg, &mut r), 0);
}

// ===========================================================================
// Attribute generation
// ===========================================================================

/// Scenario: Annotated scalar attributes of each type are generated.
/// Guarantees: Generated values have the attribute's type and respect the
/// annotation's values or range.
#[test]
fn attribute_value_scalars() {
    let mut r = rng();
    let int = &attribute(
        "server.port",
        primitive(PrimitiveOrArrayTypeSpec::Int),
        Some(
            "{ range: { min: 1024, max: 2048 }, distribution: { type: normal, mean: 1500, stddev: 300 } }",
        ),
    );
    let double = &attribute(
        "ratio",
        primitive(PrimitiveOrArrayTypeSpec::Double),
        Some("{ range: { min: 0, max: 1 } }"),
    );
    let boolean = &attribute(
        "flag",
        primitive(PrimitiveOrArrayTypeSpec::Boolean),
        Some("{ values: [false] }"),
    );
    for _ in 0..2_000 {
        match kv_scalar(&get_attribute_name_value(int, &mut r)) {
            Scalar::Int(i) => assert!((1024..=2048).contains(&i)),
            other => panic!("{other:?}"),
        }
        match kv_scalar(&get_attribute_name_value(double, &mut r)) {
            Scalar::Double(d) => assert!((0.0..=1.0).contains(&d)),
            other => panic!("{other:?}"),
        }
        assert_eq!(
            kv_scalar(&get_attribute_name_value(boolean, &mut r)),
            Scalar::Bool(false)
        );
    }
}

/// Scenario: Array and template attributes are generated from annotations.
/// Guarantees: Array lengths and items stay within their specs and template
/// keys expand to `<prefix>.<key>`.
#[test]
fn attribute_value_arrays_and_templates() {
    let mut r = rng();
    let arr = &attribute(
        "process.command_args",
        primitive(PrimitiveOrArrayTypeSpec::Strings),
        Some("{ length: { range: { min: 1, max: 4 } }, items: { values: [a, b] } }"),
    );
    let tmpl = &attribute(
        "http.request.header",
        AttributeType::Template(TemplateTypeSpec::Strings),
        Some(
            "{ keys: { values: [content-type, accept] }, value: { length: { values: [2] }, items: { values: [x] } } }",
        ),
    );
    let mut keys = HashSet::new();
    for _ in 0..500 {
        let kv = get_attribute_name_value(arr, &mut r);
        let Some(AnyValueKind::ArrayValue(a)) = kv.value.as_ref().and_then(|v| v.value.as_ref())
        else {
            panic!("expected array")
        };
        assert!((1..=4).contains(&a.values.len()));
        assert!(
            a.values
                .iter()
                .all(|v| matches!(any_scalar(v), Scalar::String(s) if s == "a" || s == "b"))
        );

        let kv = get_attribute_name_value(tmpl, &mut r);
        let _ = keys.insert(kv.key.clone());
        let Some(AnyValueKind::ArrayValue(a)) = kv.value.as_ref().and_then(|v| v.value.as_ref())
        else {
            panic!("expected array")
        };
        assert_eq!(a.values.len(), 2);
    }
    assert_eq!(
        keys,
        HashSet::from([
            "http.request.header.content-type".to_owned(),
            "http.request.header.accept".to_owned()
        ])
    );
}

/// Scenario: Enum attributes are generated with and without annotations.
/// Guarantees: Annotated enums only produce the weighted subset; unannotated
/// enums produce every member (not just the first).
#[test]
fn attribute_value_enums() {
    let mut r = rng();
    let ty = enum_type(&["GET", "POST", "DELETE", "PUT"]);
    let annotated = &attribute(
        "http.request.method",
        ty.clone(),
        Some("{ values: [GET, POST], weights: [9, 1] }"),
    );
    let plain = attribute("m", ty, None);
    let mut a = HashSet::new();
    let mut p = HashSet::new();
    for _ in 0..1_000 {
        let _ = a.insert(format!(
            "{:?}",
            kv_scalar(&get_attribute_name_value(annotated, &mut r))
        ));
        let _ = p.insert(format!(
            "{:?}",
            kv_scalar(&get_attribute_name_value(&plain, &mut r))
        ));
    }
    assert_eq!(a.len(), 2);
    assert_eq!(p.len(), 4, "unannotated enum must vary across members");
}

/// Scenario: An attribute has no annotation but has examples.
/// Guarantees: Values fall back to the examples.
#[test]
fn attribute_value_falls_back_to_examples() {
    let mut r = rng();
    let mut attr = attribute("x", primitive(PrimitiveOrArrayTypeSpec::Int), None);
    attr.examples = Some(Examples::Ints(vec![7, 9]));
    for _ in 0..100 {
        assert!(matches!(
            kv_scalar(&get_attribute_name_value(&attr, &mut r)),
            Scalar::Int(7 | 9)
        ));
    }
}

/// Scenario: An attribute carries an invalid annotation and is generated
/// anyway (startup validation would normally reject it first).
/// Guarantees: Generation does not panic and falls back to the examples.
#[test]
fn attribute_with_invalid_annotation_falls_back() {
    let mut r = rng();
    let mut attr = attribute(
        "x",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ range: { min: 0, max: 1 } }"),
    );
    attr.examples = Some(Examples::Strings(vec!["fallback".to_owned()]));
    assert_eq!(
        kv_scalar(&get_attribute_name_value(&attr, &mut r)),
        Scalar::String("fallback".to_owned())
    );
}

/// Scenario: A registry containing invalid attribute and group annotations is
/// validated.
/// Guarantees: `validate_registry` reports every invalid annotation with its
/// group, attribute, and key path, so startup fails with a complete list.
#[test]
fn validate_registry_reports_every_error() {
    let mut bad = registry().clone();
    let span = bad
        .groups
        .iter_mut()
        .find(|g| g.r#type == GroupType::Span)
        .expect("a span group");
    let mut ann = span.annotations.clone().unwrap_or_default();
    let _ = ann.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(serde_yaml::from_str("{ severity: { values: [INFO] } }").expect("yaml")),
    );
    span.annotations = Some(ann);
    let span_id = span.id.clone();
    span.attributes.push(attribute(
        "bad.attr",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ range: { min: 0, max: 1 } }"),
    ));

    let errors = annotation::validate_registry(&bad).expect_err("invalid annotations");
    assert_eq!(errors.len(), 2, "{}", annotation::format_errors(&errors));
    assert!(errors.iter().any(|e| {
        e.group == span_id
            && e.attribute.is_none()
            && e.error
                .message
                .contains("unknown key \"severity\" for span group")
    }));
    assert!(errors.iter().any(|e| e.group == span_id
        && e.attribute.as_deref() == Some("bad.attr")
        && e.error.path == format!("{}.range", annotation::NAMESPACE)));
    assert!(
        annotation::format_errors(&errors)
            .starts_with("2 invalid traffic-generator annotation(s):")
    );
}

// ===========================================================================
// Signals
// ===========================================================================

/// Scenario: Observations are bucketized with OTLP explicit bounds.
/// Guarantees: Upper-inclusive buckets, an overflow bucket, and counts that
/// sum to the number of observations.
#[test]
fn bucketize_uses_upper_inclusive_bounds() {
    let counts = bucketize(&[0.0, 1.0, 1.5, 2.0, 2.0001, 99.0], &[1.0, 2.0]);
    assert_eq!(counts, vec![2, 2, 2]);
    assert_eq!(bucketize(&[], &DEFAULT_HISTOGRAM_BOUNDS).len(), 16);
}

/// Scenario: Metrics are generated from the annotated vendored registry.
/// Guarantees: Counters are delta and non-negative, sums are delta, int
/// metrics emit `as_int` points, number values stay in their annotated range,
/// and histograms have consistent buckets, count, sum, min, and max.
#[test]
fn generated_metrics_respect_annotations() {
    let groups = groups_of(GroupType::Metric);
    let mut r = rng();
    let data = semconv_otlp_metrics_with_rng(groups.len() * 3, model(), &mut r);
    let by_name: HashMap<&str, _> = groups
        .iter()
        .map(|g| (g.metric_name.as_deref().unwrap_or_default(), *g))
        .collect();
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), groups.len() * 3);
    for metric in metrics {
        let group = by_name[metric.name.as_str()];
        let int_value = annotation::metric_value_is_int(group.annotations.as_ref());
        match metric.data.as_ref().expect("data") {
            Data::Sum(sum) => {
                assert_eq!(
                    sum.aggregation_temporality,
                    AggregationTemporality::Delta as i32
                );
                assert_eq!(
                    sum.is_monotonic,
                    group.instrument == Some(InstrumentSpec::Counter)
                );
                for p in &sum.data_points {
                    let v = number_value(p.value.as_ref(), int_value, &metric.name);
                    if sum.is_monotonic {
                        assert!(v >= 0.0, "{}: negative counter increment {v}", metric.name);
                    }
                }
            }
            Data::Gauge(g) => {
                assert!(!g.data_points.is_empty());
                for p in &g.data_points {
                    let _ = number_value(p.value.as_ref(), int_value, &metric.name);
                }
            }
            Data::Histogram(h) => {
                for p in &h.data_points {
                    assert_eq!(p.bucket_counts.len(), p.explicit_bounds.len() + 1);
                    assert_eq!(
                        p.bucket_counts.iter().sum::<u64>(),
                        p.count,
                        "{}",
                        metric.name
                    );
                    assert!(p.count >= 1);
                    let (min, max, sum) = (
                        p.min.expect("min"),
                        p.max.expect("max"),
                        p.sum.expect("sum"),
                    );
                    assert!(min <= max);
                    assert!(
                        sum >= min * p.count as f64 - 1e-6 && sum <= max * p.count as f64 + 1e-6
                    );
                }
            }
            other => panic!("unexpected data {other:?}"),
        }
    }
}

fn number_value(v: Option<&number_data_point::Value>, int: bool, name: &str) -> f64 {
    match v.expect("value set") {
        number_data_point::Value::AsInt(i) => {
            assert!(int, "{name}: double metric emitted as int");
            *i as f64
        }
        number_data_point::Value::AsDouble(d) => {
            assert!(!int, "{name}: int metric emitted as double");
            *d
        }
    }
}

/// Scenario: Spans and logs are generated from the annotated vendored
/// registry.
/// Guarantees: Span durations vary and stay non-negative, status codes come
/// from the annotation, and log severity number and text agree.
#[test]
fn generated_spans_and_logs_respect_annotations() {
    let mut r = rng();
    let traces =
        semconv_otlp_traces_with_rng(groups_of(GroupType::Span).len() * 5, model(), &mut r);
    let spans = &traces.resource_spans[0].scope_spans[0].spans;
    let durations: HashSet<u64> = spans
        .iter()
        .map(|s| s.end_time_unix_nano - s.start_time_unix_nano)
        .collect();
    assert!(durations.len() > spans.len() / 2, "durations should vary");
    let codes: HashSet<i32> = spans
        .iter()
        .filter_map(|s| s.status.as_ref().map(|s| s.code))
        .collect();
    assert!(codes.is_subset(&HashSet::from([0, 1, 2])));
    assert!(codes.contains(&2), "error status should appear");

    let logs = semconv_otlp_logs_with_rng(groups_of(GroupType::Event).len() * 10, model(), &mut r);
    let mut severities = HashSet::new();
    for rec in &logs.resource_logs[0].scope_logs[0].log_records {
        assert_eq!(
            severity_number(&rec.severity_text),
            Some(rec.severity_number)
        );
        let _ = severities.insert(rec.severity_number);
    }
    assert!(
        severities.len() > 2,
        "severities should vary: {severities:?}"
    );
}

// ===========================================================================
// Vendored registry
// ===========================================================================

/// Scenario: Every annotation in the full vendored registry is validated.
/// Guarantees: Every `traffic-generator` annotation parses and validates, so
/// the receiver starts without configuration errors.
#[test]
fn vendored_registry_annotations_all_validate() {
    if let Err(errors) = annotation::validate_registry(registry()) {
        panic!("{}", annotation::format_errors(&errors));
    }
    assert!(!groups_of(GroupType::Metric).is_empty());
    assert!(!groups_of(GroupType::Span).is_empty());
    assert!(!groups_of(GroupType::Event).is_empty());
}

/// Scenario: The vendored registry is checked for annotation coverage.
/// Guarantees: Every metric, span and event group, and every int, double,
/// boolean, or int[] attribute has a `traffic-generator` annotation.
#[test]
fn vendored_registry_annotation_coverage() {
    let mut missing = Vec::new();
    for g in &registry().groups {
        let has = |a: Option<&BTreeMap<String, YamlValue>>| {
            a.is_some_and(|a| a.contains_key(annotation::NAMESPACE))
        };
        if matches!(
            g.r#type,
            GroupType::Metric | GroupType::Span | GroupType::Event
        ) && !has(g.annotations.as_ref())
        {
            missing.push(format!("group {}", g.id));
        }
        for a in &g.attributes {
            let numeric = matches!(
                a.r#type,
                AttributeType::PrimitiveOrArray(
                    PrimitiveOrArrayTypeSpec::Int
                        | PrimitiveOrArrayTypeSpec::Double
                        | PrimitiveOrArrayTypeSpec::Boolean
                        | PrimitiveOrArrayTypeSpec::Ints
                )
            );
            if numeric && !has(a.annotations.as_ref()) {
                missing.push(format!("{} / {}", g.id, a.name));
            }
        }
    }
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "missing annotations:\n{}",
        missing.join("\n")
    );
}

/// Scenario: A config is resolved through `get_registry` for both data
/// sources.
/// Guarantees: The semantic conventions source returns the validated
/// registry and the synthetic source returns None.
#[test]
fn config_registry_paths() {
    let config = Config::new(
        TrafficConfig::new(None, None, 10, 1, 1, 1),
        default_registry_path(),
    );
    assert!(config.get_registry().expect("valid").is_some());
    let synthetic = config.with_data_source(super::config::DataSource::Synthetic);
    assert!(synthetic.get_registry().expect("valid").is_none());
}

// ===========================================================================
// Compiled model and samplers
// ===========================================================================

/// Scenario: A precompiled `ValueSampler` and the per-call `sample_value`
/// helper are driven by identically seeded RNGs for discrete, weighted, and
/// every continuous distribution (int and double output).
/// Guarantees: Compiling a sampler once and reusing it draws exactly the same
/// value sequence as rebuilding it for every draw, so the optimization does
/// not change generated values.
#[test]
fn compiled_value_sampler_matches_per_call_sampling() {
    let mut specs = vec![
        (
            ValueSpec::Discrete {
                values: vec![Scalar::Int(1), Scalar::Int(2), Scalar::Int(3)],
                weights: None,
            },
            true,
        ),
        (
            ValueSpec::Discrete {
                values: vec![Scalar::String("a".into()), Scalar::String("b".into())],
                weights: Some(vec![9.0, 1.0]),
            },
            false,
        ),
    ];
    for (_, distribution, (min, max)) in non_uniform_cases() {
        for integer in [false, true] {
            if matches!(distribution, Distribution::Zipf { .. }) && !integer {
                continue;
            }
            specs.push((
                ValueSpec::Continuous {
                    range: Range { min, max },
                    distribution,
                },
                integer,
            ));
        }
    }
    specs.push((
        ValueSpec::Continuous {
            range: Range {
                min: -5.0,
                max: 5.0,
            },
            distribution: Distribution::Uniform,
        },
        false,
    ));
    for (spec, integer) in specs {
        let compiled = ValueSampler::new(&spec, integer);
        let (mut a, mut b) = (rng(), rng());
        for _ in 0..500 {
            assert_eq!(
                compiled.sample_value(&mut a),
                sample_value(&spec, integer, &mut b),
                "{spec:?} integer={integer}"
            );
        }
    }
}

/// Scenario: The vendored registry is compiled into a `SemconvModel`.
/// Guarantees: The model has one template per span, metric, and event group
/// (in registry order) and one attribute generator per group attribute, so
/// generation cycles over exactly the same groups as the registry.
#[test]
fn compiled_model_mirrors_registry_groups() {
    let m = model();
    let spans = groups_of(GroupType::Span);
    let metrics = groups_of(GroupType::Metric);
    let events = groups_of(GroupType::Event);
    assert_eq!(m.spans.len(), spans.len());
    assert_eq!(m.metrics.len(), metrics.len());
    assert_eq!(m.events.len(), events.len());
    for (t, g) in m.spans.iter().zip(&spans) {
        assert_eq!(t.name, g.id);
        assert_eq!(t.attributes.len(), g.attributes.len());
    }
    for (t, g) in m.metrics.iter().zip(&metrics) {
        match t {
            Some(t) => {
                assert_eq!(Some(&t.name), g.metric_name.as_ref());
                assert_eq!(t.attributes.len(), g.attributes.len());
                assert_eq!(
                    t.int_value,
                    annotation::metric_value_is_int(g.annotations.as_ref())
                );
            }
            None => assert!(g.instrument.is_none()),
        }
    }
    for (t, g) in m.events.iter().zip(&events) {
        assert_eq!(Some(&t.name), g.name.as_ref());
        assert_eq!(t.attributes.len(), g.attributes.len());
    }
}

/// Scenario: A corrupted registry is compiled directly.
/// Guarantees: `SemconvModel::compile` reports the same errors as
/// `validate_registry` (group error first, then attribute errors), so
/// compiling once at startup preserves the full error report.
#[test]
fn compile_reports_same_errors_as_validation() {
    let mut bad = registry().clone();
    let event = bad
        .groups
        .iter_mut()
        .find(|g| g.r#type == GroupType::Event)
        .expect("an event group");
    let mut ann = event.annotations.clone().unwrap_or_default();
    let _ = ann.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(serde_yaml::from_str("{ severity: { values: [LOUD] } }").expect("yaml")),
    );
    event.annotations = Some(ann);
    event.attributes.push(attribute(
        "bad.attr",
        primitive(PrimitiveOrArrayTypeSpec::Int),
        Some("{ weights: [1] }"),
    ));
    let event_id = event.id.clone();

    let compiled = SemconvModel::compile(&bad).expect_err("invalid annotations");
    let validated = annotation::validate_registry(&bad).expect_err("invalid annotations");
    assert_eq!(compiled, validated);
    assert_eq!(compiled.len(), 2);
    assert_eq!(compiled[0].group, event_id);
    assert!(compiled[0].attribute.is_none());
    assert_eq!(compiled[1].attribute.as_deref(), Some("bad.attr"));
}

/// Scenario: Non-annotated attributes are compiled into fallback generators
/// for enums, example lists, single examples, and type defaults.
/// Guarantees: Compiled fallbacks produce exactly the values the
/// non-annotated rules allow: every enum member, every example, or the type
/// default.
#[test]
fn compiled_fallbacks_cover_expected_values() {
    let mut r = rng();
    let collect = |attr: &Attribute, r: &mut StdRng| -> HashSet<String> {
        let g = AttributeGen::new_or_fallback(attr);
        (0..500)
            .map(|_| format!("{:?}", kv_scalar(&g.sample(r))))
            .collect()
    };
    let e = attribute("e", enum_type(&["a", "b", "c"]), None);
    assert_eq!(collect(&e, &mut r).len(), 3);

    let mut ex = attribute("x", primitive(PrimitiveOrArrayTypeSpec::String), None);
    ex.examples = Some(Examples::Strings(vec!["p".into(), "q".into()]));
    assert_eq!(
        collect(&ex, &mut r),
        HashSet::from([
            format!("{:?}", Scalar::String("p".into())),
            format!("{:?}", Scalar::String("q".into()))
        ])
    );

    let d = attribute("d", primitive(PrimitiveOrArrayTypeSpec::Double), None);
    assert_eq!(
        collect(&d, &mut r),
        HashSet::from([format!("{:?}", Scalar::Double(3.13))])
    );
    let b = attribute("b", primitive(PrimitiveOrArrayTypeSpec::Boolean), None);
    assert_eq!(
        collect(&b, &mut r),
        HashSet::from([format!("{:?}", Scalar::Bool(true))])
    );
}

/// Scenario: Signals are generated from the same compiled model twice with
/// identically seeded RNGs.
/// Guarantees: The compiled model is reused without mutation, so generation
/// is reproducible for a seed and repeated batches do not depend on prior
/// generation.
#[test]
fn compiled_model_generation_is_reproducible() {
    let (mut a, mut b) = (rng(), rng());
    let strip = |mut d: otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData| {
        for rl in &mut d.resource_logs {
            for sl in &mut rl.scope_logs {
                for rec in &mut sl.log_records {
                    rec.time_unix_nano = 0;
                    rec.observed_time_unix_nano = 0;
                }
            }
        }
        d
    };
    let n = groups_of(GroupType::Event).len() * 2;
    assert_eq!(
        strip(semconv_otlp_logs_with_rng(n, model(), &mut a)),
        strip(semconv_otlp_logs_with_rng(n, model(), &mut b))
    );
}
