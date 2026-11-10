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
    AggregationTemporality, HistogramDataPoint, Metric, metric::Data, number_data_point,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde_json::{Map, Value};
use weaver_forge::v1::registry::ResolvedRegistry;
use weaver_resolved_schema::v1::attribute::Attribute;
use weaver_semconv::YamlValue;
use weaver_semconv::v1::attribute::{
    AttributeType, EnumEntriesSpec, Examples, PrimitiveOrArrayTypeSpec, TemplateTypeSpec,
    ValueSpec as SemconvValueSpec,
};
use weaver_semconv::v1::group::{GroupType, InstrumentSpec};

use super::annotation::{
    self, AnnotationError, AttributeSpec, Distribution, Range, Scalar, ScalarKind, ValueSpec,
};
use super::attributes::AttributeGen;
use super::config::{Config, TrafficConfig, vendored_registry_path};
use super::sampler::{
    IndexSampler, NumberSampler, ValueSampler, lognormal_params, normal_cdf, normal_inverse_sf,
    normal_quantile, normal_sf,
};
use super::semconv_model::{SemconvModel, severity_entry, status_entry};
use super::semconv_signal::{
    bucket_index, semconv_otlp_logs_with_rng, semconv_otlp_metrics_with_rng,
    semconv_otlp_traces_with_rng,
};
use super::synthetic_signal::HISTOGRAM_BOUNDS;

// ===========================================================================
// Helpers
// ===========================================================================

const SEED: u64 = 0x5eed_cafe;

fn rng() -> StdRng {
    StdRng::seed_from_u64(SEED)
}

type Mapping = Map<String, Value>;

/// Parse a YAML fixture into a `serde_json` object.
///
/// `YamlValue` wraps a [`serde_json::Value`], so fixtures authored in YAML are
/// transcoded into JSON before use.
fn yaml_from_str(src: &str) -> Value {
    let yaml: serde_yaml::Value = serde_yaml::from_str(src).expect("valid yaml");
    serde_json::to_value(yaml).expect("yaml convertible to json")
}

fn yaml_map(src: &str) -> Mapping {
    match yaml_from_str(src) {
        Value::Object(m) => m,
        other => panic!("expected mapping, got {other:?}"),
    }
}

fn parse_scalar(src: &str, kind: ScalarKind) -> Result<ValueSpec, AnnotationError> {
    annotation::parse_value_spec(&yaml_map(src), annotation::NAMESPACE, kind)
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
                YamlValue(yaml_from_str(src)),
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

/// Compile `attribute` (its annotation must be valid) and draw one pair.
fn gen_kv(attribute: &Attribute, rng: &mut StdRng) -> KeyValue {
    AttributeGen::try_new(attribute)
        .unwrap_or_else(|e| panic!("{}: {e}", attribute.name))
        .sample(rng)
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
            vendored_registry_path(),
        );
        config
            .resolve_registry()
            .expect("vendored registry resolves")
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
        "must be in 0..=1024",
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
/// Guarantees: All three histogram keys are captured, including bounds.
#[test]
fn parse_histogram_metric_spec() {
    let spec = annotation::parse_metric_spec(
        &yaml_map(
            "value: { range: { min: 0.001, max: 10 }, distribution: { type: lognormal, mean: 0.2, stddev: 0.5 } }\n\
             observations: { range: { min: 10, max: 1000 } }\n\
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
        "must be in 1..=1000",
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

/// Scenario: Severity and status names are mapped to OTLP numbers.
/// Guarantees: Every documented name maps to its protobuf value.
#[test]
fn severity_and_status_mappings() {
    for (i, name) in annotation::SEVERITY_NAMES.iter().enumerate() {
        assert_eq!(severity_entry(name), Some((i as i32 + 1, *name)));
    }
    assert_eq!(severity_entry("LOUD"), None);
    assert_eq!(status_entry("unset"), Some(0));
    assert_eq!(status_entry("ok"), Some(1));
    assert_eq!(status_entry("error"), Some(2));
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
        (1.0, 1e-9),
    ] {
        let (mu, sigma) = lognormal_params(mean_v, sd_v);
        let s2 = sigma * sigma;
        let mean = (mu + s2 / 2.0).exp();
        // Use exp_m1 so exp(s2) - 1 keeps precision for tiny s2; otherwise the
        // reconstructed stddev for small variance ratios would itself round to
        // zero and the assertion would spuriously fail.
        let sd = (s2.exp_m1() * (2.0 * mu + s2).exp()).sqrt();
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

/// Scenario: A lognormal annotation requests a tiny but nonzero stddev
/// (mean 1, stddev 1e-9) whose variance ratio, added to 1.0, rounds back to
/// 1.0 under naive arithmetic.
/// Guarantees: The underlying sigma stays strictly positive and sampling over
/// the valid range does not collapse every draw to the mean, so small
/// requested variances are preserved rather than silently dropped.
#[test]
fn lognormal_tiny_variance_is_preserved() {
    let (_, sigma) = lognormal_params(1.0, 1e-9);
    assert!(sigma > 0.0, "sigma collapsed to {sigma}");

    let spec = parse_scalar(
        "range: { min: 0, max: 2 }\n\
         distribution: { type: lognormal, mean: 1, stddev: 1e-9 }",
        ScalarKind::Double,
    )
    .expect("tiny-variance lognormal is valid");
    let sampler = ValueSampler::new(&spec, false);
    let mut r = rng();
    let mut distinct = false;
    for _ in 0..1_000 {
        let x = sampler.sample_f64(&mut r);
        assert!((0.0..=2.0).contains(&x), "out of range: {x}");
        if (x - 1.0).abs() > 0.0 {
            distinct = true;
        }
    }
    assert!(distinct, "all draws collapsed to the mean");
}

/// Scenario: A lognormal annotation uses an extreme scale (mean and stddev
/// both 1e200) where squaring mean and stddev independently would overflow to
/// infinity and yield inf/inf == NaN.
/// Guarantees: The spec compiles into a finite sigma and is accepted by range
/// validation instead of being rejected with a spurious "~0 probability"
/// error.
#[test]
fn lognormal_extreme_scale_is_accepted() {
    let (mu, sigma) = lognormal_params(1e200, 1e200);
    assert!(mu.is_finite() && sigma.is_finite(), "mu {mu} sigma {sigma}");

    let _ = parse_scalar(
        "range: { min: 0, max: 1e300 }\n\
         distribution: { type: lognormal, mean: 1e200, stddev: 1e200 }",
        ScalarKind::Double,
    )
    .expect("extreme-scale lognormal is valid");
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

/// Scenario: `ValueSampler::sample_value` and `ValueSampler::sample_count`
/// are called on samplers compiled from continuous and discrete specs.
/// Guarantees: Continuous int samplers yield `Scalar::Int`, double samplers
/// yield `Scalar::Double`, discrete values are returned as written, and
/// `sample_count` never goes negative.
#[test]
fn sample_value_output_types() {
    let spec = ValueSpec::Continuous {
        range: Range { min: 0.0, max: 9.0 },
        distribution: Distribution::Uniform,
    };
    let mut r = rng();
    assert!(matches!(
        ValueSampler::new(&spec, true).sample_value(&mut r),
        Scalar::Int(_)
    ));
    assert!(matches!(
        ValueSampler::new(&spec, false).sample_value(&mut r),
        Scalar::Double(_)
    ));
    let neg = ValueSampler::new(
        &ValueSpec::Discrete {
            values: vec![Scalar::Int(-3)],
            weights: None,
        },
        true,
    );
    assert_eq!(neg.sample_value(&mut r), Scalar::Int(-3));
    assert_eq!(neg.sample_count(&mut r), 0);
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
        match kv_scalar(&gen_kv(int, &mut r)) {
            Scalar::Int(i) => assert!((1024..=2048).contains(&i)),
            other => panic!("{other:?}"),
        }
        match kv_scalar(&gen_kv(double, &mut r)) {
            Scalar::Double(d) => assert!((0.0..=1.0).contains(&d)),
            other => panic!("{other:?}"),
        }
        assert_eq!(kv_scalar(&gen_kv(boolean, &mut r)), Scalar::Bool(false));
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
        let kv = gen_kv(arr, &mut r);
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

        let kv = gen_kv(tmpl, &mut r);
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
/// enums always produce the first member, matching the behavior before
/// annotations were introduced.
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
        let _ = a.insert(format!("{:?}", kv_scalar(&gen_kv(annotated, &mut r))));
        let _ = p.insert(format!("{:?}", kv_scalar(&gen_kv(&plain, &mut r))));
    }
    assert_eq!(a.len(), 2);
    assert_eq!(
        p,
        HashSet::from([format!("{:?}", Scalar::String("GET".into()))]),
        "unannotated enum emits the first member"
    );
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
            kv_scalar(&gen_kv(&attr, &mut r)),
            Scalar::Int(7 | 9)
        ));
    }
}

/// Scenario: An attribute that has examples carries an invalid annotation (a
/// continuous range on a string attribute) and is compiled.
/// Guarantees: Compilation fails with the annotation key path instead of
/// silently falling back to the examples, so an invalid annotation always
/// fails receiver startup.
#[test]
fn attribute_with_invalid_annotation_falls_back() {
    let mut attr = attribute(
        "x",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ range: { min: 0, max: 1 } }"),
    );
    attr.examples = Some(Examples::Strings(vec!["fallback".to_owned()]));
    expect_err(
        AttributeGen::try_new(&attr),
        &format!("{}.range", annotation::NAMESPACE),
        "continuous form is not allowed for string",
    );
}

/// Scenario: A registry containing invalid attribute and group annotations is
/// validated.
/// Guarantees: `SemconvModel::compile` reports every invalid annotation with
/// its group, attribute, and key path, so startup fails with a complete list.
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
        YamlValue(yaml_from_str("{ severity: { values: [INFO] } }")),
    );
    span.annotations = Some(ann);
    let span_id = span.id.clone();
    span.attributes.push(attribute(
        "bad.attr",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ range: { min: 0, max: 1 } }"),
    ));

    let errors = SemconvModel::compile(&bad).expect_err("invalid annotations");
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

/// Scenario: Observations are assigned to OTLP explicit buckets.
/// Guarantees: Buckets are upper-inclusive and values above the last bound
/// land in the overflow bucket at index `bounds.len()`.
#[test]
fn bucket_index_uses_upper_inclusive_bounds() {
    let bounds = [1.0, 2.0];
    let indexes: Vec<usize> = [0.0, 1.0, 1.5, 2.0, 2.0001, 99.0]
        .iter()
        .map(|x| bucket_index(&bounds, *x))
        .collect();
    assert_eq!(indexes, vec![0, 0, 1, 1, 2, 2]);
    assert_eq!(
        bucket_index(HISTOGRAM_BOUNDS, f64::MAX),
        HISTOGRAM_BOUNDS.len()
    );
}

/// Scenario: Metrics are generated from every metric group in the vendored
/// registry, annotated or not.
/// Guarantees: The batch holds exactly the requested number of metrics,
/// counters are non-negative, sums and histograms carry the template's
/// temporality (`Unspecified` unless annotated, with a start time only for
/// delta), every number point is `as_double`, and histograms have consistent
/// buckets, count, sum, min, and max.
#[test]
fn generated_metrics_are_well_formed() {
    let groups = groups_of(GroupType::Metric);
    let requested = groups.len() * 3;
    let mut r = rng();
    let data = semconv_otlp_metrics_with_rng(requested, model(), &mut r);
    let by_name: HashMap<&str, _> = groups
        .iter()
        .map(|g| (g.metric_name.as_deref().unwrap_or_default(), *g))
        .collect();
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), requested);
    let templates: HashMap<&str, _> = model()
        .metrics
        .iter()
        .map(|t| (t.name.as_str(), t))
        .collect();
    for metric in metrics {
        let group = by_name[metric.name.as_str()];
        let temporality = templates[metric.name.as_str()].temporality;
        let delta = temporality == AggregationTemporality::Delta;
        match metric.data.as_ref().expect("data") {
            Data::Sum(sum) => {
                assert_eq!(sum.aggregation_temporality, temporality as i32);
                for p in &sum.data_points {
                    assert_eq!(p.start_time_unix_nano > 0, delta, "{}", metric.name);
                }
                assert_eq!(
                    sum.is_monotonic,
                    group.instrument == Some(InstrumentSpec::Counter)
                );
                for p in &sum.data_points {
                    let v = number_value(p.value.as_ref(), &metric.name);
                    if sum.is_monotonic {
                        assert!(v >= 0.0, "{}: negative counter increment {v}", metric.name);
                    }
                }
            }
            Data::Gauge(g) => {
                assert!(!g.data_points.is_empty());
                for p in &g.data_points {
                    let _ = number_value(p.value.as_ref(), &metric.name);
                }
            }
            Data::Histogram(h) => {
                assert_eq!(h.aggregation_temporality, temporality as i32);
                for p in &h.data_points {
                    assert_histogram_point_consistent(p, &metric.name);
                    assert_eq!(p.start_time_unix_nano > 0, delta, "{}", metric.name);
                }
            }
            other => panic!("unexpected data {other:?}"),
        }
    }
}

/// The value of a number point, which the generator always emits as a double.
fn number_value(v: Option<&number_data_point::Value>, name: &str) -> f64 {
    match v.expect("value set") {
        number_data_point::Value::AsDouble(d) => *d,
        number_data_point::Value::AsInt(i) => panic!("{name}: number point emitted as int {i}"),
    }
}

/// Number of data points in a generated metric.
fn point_count(metric: &Metric) -> usize {
    match metric.data.as_ref().expect("data") {
        Data::Sum(s) => s.data_points.len(),
        Data::Gauge(g) => g.data_points.len(),
        Data::Histogram(h) => h.data_points.len(),
        other => panic!("unexpected data {other:?}"),
    }
}

fn assert_histogram_point_consistent(p: &HistogramDataPoint, name: &str) {
    assert_eq!(p.bucket_counts.len(), p.explicit_bounds.len() + 1, "{name}");
    assert_eq!(p.bucket_counts.iter().sum::<u64>(), p.count, "{name}");
    assert!(p.count >= 1, "{name}");
    let (min, max, sum) = (
        p.min.expect("min"),
        p.max.expect("max"),
        p.sum.expect("sum"),
    );
    assert!(min <= max, "{name}");
    assert!(
        sum >= min * p.count as f64 - 1e-6 && sum <= max * p.count as f64 + 1e-6,
        "{name}"
    );
}

// ===========================================================================
// Vendored registry: representative annotations
// ===========================================================================
//
// The vendored registry carries one representative `traffic-generator`
// annotation per supported annotation kind (see the receiver README). The
// tests below generate signals from those groups only and check that each
// annotation drives the generated values.

/// Span group annotated with `duration` and `status`; its attributes
/// carry the int, double, enum, string, array, and template annotations.
const REPRESENTATIVE_SPAN: &str = "span.azure.cosmosdb.client";
/// Histogram metric annotated with `value`, `bounds`, and `observations`.
const REPRESENTATIVE_HISTOGRAM: &str = "jvm.gc.duration";
/// Counter metric annotated with an exponential `value`.
const REPRESENTATIVE_COUNTER: &str = "jvm.class.loaded";
/// UpDownCounter metric annotated with a uniform `value`; carries the
/// annotated boolean attribute `jvm.thread.daemon`.
const REPRESENTATIVE_UPDOWNCOUNTER: &str = "jvm.thread.count";
/// Gauge metric annotated with a discrete double `value`.
const REPRESENTATIVE_GAUGE: &str = "jvm.cpu.recent_utilization";
/// Event group annotated with `severity`; carries the annotated `any`
/// attribute `feature_flag.result.value`.
const REPRESENTATIVE_EVENT: &str = "feature_flag.evaluation";

/// Group-level and attribute-level representative annotations, as
/// `(group id, attribute name)`; `None` marks a group-level annotation.
const REPRESENTATIVE_ANNOTATIONS: &[(&str, Option<&str>)] = &[
    ("span.azure.cosmosdb.client", None),
    (
        "span.azure.cosmosdb.client",
        Some("azure.cosmosdb.request.body.size"),
    ),
    (
        "span.azure.cosmosdb.client",
        Some("azure.cosmosdb.operation.request_charge"),
    ),
    (
        "span.azure.cosmosdb.client",
        Some("azure.cosmosdb.connection.mode"),
    ),
    (
        "span.azure.cosmosdb.client",
        Some("azure.cosmosdb.operation.contacted_regions"),
    ),
    ("span.azure.cosmosdb.client", Some("db.operation.name")),
    (
        "span.azure.cosmosdb.client",
        Some("db.response.returned_rows"),
    ),
    ("span.azure.cosmosdb.client", Some("db.query.parameter")),
    ("metric.jvm.gc.duration", None),
    ("metric.jvm.class.loaded", None),
    ("metric.jvm.thread.count", None),
    ("metric.jvm.thread.count", Some("jvm.thread.daemon")),
    ("metric.jvm.cpu.recent_utilization", None),
    ("event.feature_flag.evaluation", None),
    (
        "event.feature_flag.evaluation",
        Some("feature_flag.result.value"),
    ),
];

/// Value of attribute `key`, panicking when it is missing.
fn attr_value<'a>(attrs: &'a [KeyValue], key: &str) -> &'a AnyValue {
    attrs
        .iter()
        .find(|kv| kv.key == key)
        .and_then(|kv| kv.value.as_ref())
        .unwrap_or_else(|| panic!("attribute {key} missing"))
}

fn attr_scalar(attrs: &[KeyValue], key: &str) -> Scalar {
    any_scalar(attr_value(attrs, key))
}

fn attr_int(attrs: &[KeyValue], key: &str) -> i64 {
    match attr_scalar(attrs, key) {
        Scalar::Int(i) => i,
        other => panic!("{key}: expected int, got {other:?}"),
    }
}

fn attr_string(attrs: &[KeyValue], key: &str) -> String {
    match attr_scalar(attrs, key) {
        Scalar::String(s) => s,
        other => panic!("{key}: expected string, got {other:?}"),
    }
}

/// A model restricted to the named span, metric, and event templates.
fn representative_model(spans: &[&str], metrics: &[&str], events: &[&str]) -> SemconvModel {
    let m = model();
    let model = SemconvModel {
        spans: m
            .spans
            .iter()
            .filter(|t| spans.contains(&t.name.as_str()))
            .cloned()
            .collect(),
        metrics: m
            .metrics
            .iter()
            .filter(|t| metrics.contains(&t.name.as_str()))
            .cloned()
            .collect(),
        events: m
            .events
            .iter()
            .filter(|t| events.contains(&t.name.as_str()))
            .cloned()
            .collect(),
    };
    assert_eq!(model.spans.len(), spans.len(), "missing span templates");
    assert_eq!(
        model.metrics.len(),
        metrics.len(),
        "missing metric templates"
    );
    assert_eq!(model.events.len(), events.len(), "missing event templates");
    model
}

/// Scenario: The vendored registry is resolved and every representative
/// group and attribute is looked up.
/// Guarantees: Each representative annotation (one per supported annotation
/// kind) is still present, so a registry refresh that drops them fails here
/// with the missing group or attribute instead of silently weakening the
/// generation tests below.
#[test]
fn vendored_registry_keeps_representative_annotations() {
    let has = |a: Option<&BTreeMap<String, YamlValue>>| {
        a.is_some_and(|a| a.contains_key(annotation::NAMESPACE))
    };
    let mut missing = Vec::new();
    for (group_id, attribute) in REPRESENTATIVE_ANNOTATIONS {
        let Some(group) = registry().groups.iter().find(|g| g.id == *group_id) else {
            missing.push(format!("group {group_id} not found"));
            continue;
        };
        let annotated = match attribute {
            None => has(group.annotations.as_ref()),
            Some(name) => group
                .attributes
                .iter()
                .find(|a| a.name == *name)
                .is_some_and(|a| has(a.annotations.as_ref())),
        };
        if !annotated {
            missing.push(format!("{group_id} / {}", attribute.unwrap_or("(group)")));
        }
    }
    assert!(
        missing.is_empty(),
        "missing representative annotations:\n{}",
        missing.join("\n")
    );
}

/// Scenario: Every annotation in the vendored registry is compiled.
/// Guarantees: Every `traffic-generator` annotation parses and validates, so
/// the receiver starts without configuration errors when `registry_path`
/// points at the vendored registry.
#[test]
fn vendored_registry_annotations_all_validate() {
    if let Err(errors) = SemconvModel::compile(registry()) {
        panic!("{}", annotation::format_errors(&errors));
    }
    assert!(!groups_of(GroupType::Metric).is_empty());
    assert!(!groups_of(GroupType::Span).is_empty());
    assert!(!groups_of(GroupType::Event).is_empty());
}

/// Scenario: Spans are generated from the representative span group, whose
/// group annotation sets `duration` and `status` and whose attribute
/// definitions are annotated with an int lognormal range, a double normal
/// range, a weighted enum, an unweighted string list, an int zipf range, a
/// string array, and a string template.
/// Guarantees: Durations stay within the annotated range and vary, every
/// annotated status appears, and each attribute value has the attribute's
/// type and stays within its annotated range or value set (zipf favors the
/// range minimum, the enum favors its heavier member, template names expand
/// to `<prefix>.<key>`).
#[test]
fn representative_span_respects_annotations() {
    const N: usize = 2_000;
    let model = representative_model(&[REPRESENTATIVE_SPAN], &[], &[]);
    let traces = semconv_otlp_traces_with_rng(N, &model, &mut rng());
    let spans = &traces.resource_spans[0].scope_spans[0].spans;
    assert_eq!(spans.len(), N);

    let mut durations = HashSet::new();
    let mut codes = HashSet::new();
    let mut modes: HashMap<String, usize> = HashMap::new();
    let mut operations = HashSet::new();
    let mut rows: HashMap<i64, usize> = HashMap::new();
    let mut region_lengths = HashSet::new();
    let mut parameter_keys = HashSet::new();
    for span in spans {
        let d = span.end_time_unix_nano - span.start_time_unix_nano;
        assert!(
            (100_000..=30_000_000_000).contains(&d),
            "duration {d}ns outside [0.1ms, 30000ms]"
        );
        let _ = durations.insert(d);
        let _ = codes.insert(span.status.as_ref().expect("annotated status").code);

        let a = &span.attributes;
        let size = attr_int(a, "azure.cosmosdb.request.body.size");
        assert!((0..=1_048_576).contains(&size), "body size {size}");
        match attr_scalar(a, "azure.cosmosdb.operation.request_charge") {
            Scalar::Double(c) => assert!((1.0..=100.0).contains(&c), "charge {c}"),
            other => panic!("request_charge: expected double, got {other:?}"),
        }
        *modes
            .entry(attr_string(a, "azure.cosmosdb.connection.mode"))
            .or_default() += 1;
        let _ = operations.insert(attr_string(a, "db.operation.name"));
        let n = attr_int(a, "db.response.returned_rows");
        assert!((1..=1000).contains(&n), "returned_rows {n}");
        *rows.entry(n).or_default() += 1;

        let Some(AnyValueKind::ArrayValue(regions)) =
            attr_value(a, "azure.cosmosdb.operation.contacted_regions")
                .value
                .as_ref()
        else {
            panic!("contacted_regions: expected array")
        };
        let _ = region_lengths.insert(regions.values.len());
        for item in &regions.values {
            assert!(
                matches!(any_scalar(item), Scalar::String(s)
                    if ["East US", "West Europe", "Southeast Asia"].contains(&s.as_str())),
                "unexpected region {item:?}"
            );
        }

        let params: Vec<&KeyValue> = a
            .iter()
            .filter(|kv| kv.key.starts_with("db.query.parameter."))
            .collect();
        assert_eq!(params.len(), 1, "one template attribute per span");
        let _ = parameter_keys.insert(params[0].key.clone());
        assert!(
            matches!(kv_scalar(params[0]), Scalar::String(s)
                if ["42", "alice", "100"].contains(&s.as_str())),
            "unexpected parameter value {:?}",
            params[0].value
        );
    }

    assert!(durations.len() > N / 2, "durations should vary");
    assert_eq!(codes, HashSet::from([0, 1, 2]));
    assert_eq!(
        modes.keys().cloned().collect::<HashSet<_>>(),
        HashSet::from(["direct".to_owned(), "gateway".to_owned()])
    );
    assert!(modes["direct"] > modes["gateway"], "weights 9:1 {modes:?}");
    assert_eq!(
        operations,
        HashSet::from([
            "ReadItem".to_owned(),
            "CreateItem".to_owned(),
            "QueryItems".to_owned()
        ])
    );
    let most_frequent_rows = rows.iter().max_by_key(|(_, c)| **c).map(|(v, _)| *v);
    assert_eq!(most_frequent_rows, Some(1), "zipf favors range.min");
    assert_eq!(region_lengths, HashSet::from([1, 2, 3]));
    assert_eq!(
        parameter_keys,
        HashSet::from([
            "db.query.parameter.id".to_owned(),
            "db.query.parameter.name".to_owned(),
            "db.query.parameter.limit".to_owned()
        ])
    );
}

/// Scenario: Metrics are generated from the representative histogram,
/// counter, updowncounter, and gauge groups, and the updowncounter carries
/// an annotated boolean attribute.
/// Guarantees: The histogram uses the annotated `bounds`, per-point
/// observation counts, value range, and delta `temporality` (with a start
/// time no later than the point time), with one data point per metric; the
/// counter and updowncounter, which have no `temporality` annotation, keep
/// `Unspecified` temporality; the
/// counter emits monotonic increments within its exponential range; the
/// updowncounter emits non-monotonic values within its uniform range with a
/// weighted boolean attribute; the gauge emits only the annotated doubles;
/// the batch holds exactly the requested number of metrics.
#[test]
fn representative_metrics_respect_annotations() {
    const ROUNDS: usize = 200;
    let names = [
        REPRESENTATIVE_HISTOGRAM,
        REPRESENTATIVE_COUNTER,
        REPRESENTATIVE_UPDOWNCOUNTER,
        REPRESENTATIVE_GAUGE,
    ];
    let model = representative_model(&[], &names, &[]);
    let data = semconv_otlp_metrics_with_rng(names.len() * ROUNDS, &model, &mut rng());
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), names.len() * ROUNDS);

    let mut histogram_points = HashSet::new();
    let mut daemon: HashMap<bool, usize> = HashMap::new();
    let mut gauge_values = HashSet::new();
    let mut counter_values = HashSet::new();
    for metric in metrics {
        let name = metric.name.as_str();
        match (name, metric.data.as_ref().expect("data")) {
            (REPRESENTATIVE_HISTOGRAM, Data::Histogram(h)) => {
                assert_eq!(
                    h.aggregation_temporality,
                    AggregationTemporality::Delta as i32
                );
                let _ = histogram_points.insert(h.data_points.len());
                for p in &h.data_points {
                    assert!(
                        p.start_time_unix_nano > 0 && p.start_time_unix_nano <= p.time_unix_nano,
                        "delta start time {} vs time {}",
                        p.start_time_unix_nano,
                        p.time_unix_nano
                    );
                    assert_histogram_point_consistent(p, name);
                    assert_eq!(p.explicit_bounds, vec![0.01, 0.1, 1.0, 10.0]);
                    assert!((10..=100).contains(&p.count), "observations {}", p.count);
                    assert!(p.min.expect("min") >= 0.0 && p.max.expect("max") <= 10.0);
                }
            }
            (REPRESENTATIVE_COUNTER, Data::Sum(s)) => {
                assert!(s.is_monotonic);
                assert_eq!(
                    s.aggregation_temporality,
                    AggregationTemporality::Unspecified as i32
                );
                for p in &s.data_points {
                    let v = number_value(p.value.as_ref(), name);
                    assert!((0.0..=100.0).contains(&v), "{name} value {v}");
                    let _ = counter_values.insert(v as i64);
                }
            }
            (REPRESENTATIVE_UPDOWNCOUNTER, Data::Sum(s)) => {
                assert!(!s.is_monotonic);
                assert_eq!(
                    s.aggregation_temporality,
                    AggregationTemporality::Unspecified as i32
                );
                for p in &s.data_points {
                    let v = number_value(p.value.as_ref(), name);
                    assert!((10.0..=200.0).contains(&v), "{name} value {v}");
                    match attr_scalar(&p.attributes, "jvm.thread.daemon") {
                        Scalar::Bool(b) => *daemon.entry(b).or_default() += 1,
                        other => panic!("jvm.thread.daemon: expected bool, got {other:?}"),
                    }
                }
            }
            (REPRESENTATIVE_GAUGE, Data::Gauge(g)) => {
                for p in &g.data_points {
                    let v = number_value(p.value.as_ref(), name);
                    let _ = gauge_values.insert(v.to_bits());
                }
            }
            (_, other) => panic!("{name}: unexpected data {other:?}"),
        }
    }

    assert_eq!(histogram_points, HashSet::from([1]));
    assert!(counter_values.len() > 5, "counter values should vary");
    assert!(
        daemon[&true] > daemon[&false],
        "weights 3:1 favor true {daemon:?}"
    );
    assert_eq!(
        gauge_values,
        [0.25_f64, 0.5, 0.75].iter().map(|v| v.to_bits()).collect()
    );
}

/// Scenario: Logs are generated from the representative event group, whose
/// group annotation sets `severity` and whose `any` attribute definition is
/// annotated with mixed string, boolean, int, and double values.
/// Guarantees: Severity number and text agree and cover exactly the annotated
/// names, and the `any` attribute emits every annotated value with its own
/// scalar type.
#[test]
fn representative_event_respects_annotations() {
    const N: usize = 1_000;
    let model = representative_model(&[], &[], &[REPRESENTATIVE_EVENT]);
    let logs = semconv_otlp_logs_with_rng(N, &model, &mut rng());
    let records = &logs.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(records.len(), N);

    let mut severities = HashSet::new();
    let mut values = HashSet::new();
    for rec in records {
        assert_eq!(
            severity_entry(&rec.severity_text).map(|(n, _)| n),
            Some(rec.severity_number)
        );
        let _ = severities.insert(rec.severity_text.clone());
        let _ = values.insert(format!(
            "{:?}",
            attr_scalar(&rec.attributes, "feature_flag.result.value")
        ));
    }
    assert_eq!(
        severities,
        HashSet::from(["INFO".to_owned(), "DEBUG".to_owned(), "WARN".to_owned()])
    );
    assert_eq!(
        values,
        [
            Scalar::String("#ff0000".into()),
            Scalar::Bool(true),
            Scalar::Int(3),
            Scalar::Double(0.5),
        ]
        .iter()
        .map(|s| format!("{s:?}"))
        .collect()
    );
}

/// Scenario: A config is compiled through `get_semconv_model` for both data
/// sources.
/// Guarantees: The semantic conventions source returns the validated model
/// compiled from the vendored registry, and the synthetic source returns None.
#[test]
fn config_registry_paths() {
    let config = Config::new(
        TrafficConfig::new(None, None, 10, 1, 1, 1),
        vendored_registry_path(),
    )
    .with_annotations(true);
    let compiled = config
        .get_semconv_model()
        .expect("valid")
        .expect("semantic conventions data source");
    assert_eq!(compiled.spans.len(), model().spans.len());
    let synthetic = config.with_data_source(super::config::DataSource::Synthetic);
    assert!(synthetic.get_semconv_model().expect("valid").is_none());
}

// ===========================================================================
// Annotations disabled
// ===========================================================================

/// Compile the vendored registry with annotations disabled.
fn unannotated_model() -> SemconvModel {
    SemconvModel::compile_without_annotations(registry())
}

/// Scenario: `get_semconv_model` is called on the vendored, annotated
/// registry with `annotations` disabled (the default) and enabled.
/// Guarantees: Both produce the same groups, but only the enabled model
/// carries annotation-driven samplers, so the flag alone selects whether
/// annotations drive generation.
#[test]
fn config_annotations_flag_selects_compilation() {
    let config = Config::new(
        TrafficConfig::new(None, None, 10, 1, 1, 1),
        vendored_registry_path(),
    );
    assert!(!config.annotations());
    let disabled = config
        .clone()
        .get_semconv_model()
        .expect("valid")
        .expect("semantic conventions data source");
    let enabled = config
        .with_annotations(true)
        .get_semconv_model()
        .expect("valid")
        .expect("semantic conventions data source");

    assert_eq!(disabled.spans.len(), enabled.spans.len());
    assert_eq!(disabled.metrics.len(), enabled.metrics.len());
    assert_eq!(disabled.events.len(), enabled.events.len());
    assert!(enabled.spans.iter().any(|s| s.status.is_some()));
    assert!(enabled.metrics.iter().any(|m| m.value.is_some()));
    assert!(enabled.events.iter().any(|e| e.severity.is_some()));
    assert!(disabled.spans.iter().all(|s| s.status.is_none()));
    assert!(disabled.spans.iter().all(|s| s.duration_ms.is_none()));
    assert!(disabled.metrics.iter().all(|m| m.value.is_none()));
    assert!(disabled.metrics.iter().all(|m| m.observations.is_none()));
    assert!(disabled.events.iter().all(|e| e.severity.is_none()));
}

/// Scenario: A registry whose span group and attribute carry invalid
/// `traffic-generator` annotations is compiled with and without annotations.
/// Guarantees: With annotations disabled, compilation never reads the
/// annotations and succeeds with the invalid attribute kept as a fallback
/// generator; with annotations enabled, the same registry still fails.
#[test]
fn compile_without_annotations_ignores_invalid_annotations() {
    let mut bad = registry().clone();
    let span = bad
        .groups
        .iter_mut()
        .find(|g| g.r#type == GroupType::Span)
        .expect("a span group");
    let mut ann = span.annotations.clone().unwrap_or_default();
    let _ = ann.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(yaml_from_str("{ severity: { values: [INFO] } }")),
    );
    span.annotations = Some(ann);
    let span_id = span.id.clone();
    let attr_count = span.attributes.len() + 1;
    span.attributes.push(attribute(
        "bad.attr",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ range: { min: 0, max: 1 } }"),
    ));
    // A non-signal group with an annotation is rejected only when enabled.
    let attr_group = bad
        .groups
        .iter_mut()
        .find(|g| g.r#type == GroupType::AttributeGroup)
        .expect("an attribute group");
    let mut ann = attr_group.annotations.clone().unwrap_or_default();
    let _ = ann.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(yaml_from_str("{ value: { values: [1] } }")),
    );
    attr_group.annotations = Some(ann);

    assert!(SemconvModel::compile(&bad).is_err());
    let model = SemconvModel::compile_without_annotations(&bad);
    let span = model
        .spans
        .iter()
        .find(|s| s.name == span_id)
        .expect("span compiled");
    assert_eq!(span.attributes.len(), attr_count);
    let kv = span.attributes.last().expect("bad.attr").sample(&mut rng());
    assert_eq!(kv.key, "bad.attr");
    assert_eq!(kv_scalar(&kv), Scalar::String("value".into()));
}

/// Scenario: Spans, metrics, and logs are generated from the vendored,
/// annotated registry with annotations disabled.
/// Guarantees: Output matches the pre-annotation generator: spans last the
/// fixed default delay with no status, logs are INFO with no severity text,
/// number points are 1.0 with `Unspecified` temporality and no start time,
/// histogram points carry no buckets, bounds, count, sum, min, or max, and
/// annotated attributes use their non-annotated fallback values.
#[test]
fn generation_without_annotations_matches_unannotated_output() {
    use super::fake_data::delay;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::SeverityNumber;

    let model = unannotated_model();
    let mut r = rng();

    let n_spans = model.spans.len() * 2;
    let traces = semconv_otlp_traces_with_rng(n_spans, &model, &mut r);
    let spans = &traces.resource_spans[0].scope_spans[0].spans;
    assert_eq!(spans.len(), n_spans);
    for span in spans {
        assert_eq!(
            span.end_time_unix_nano - span.start_time_unix_nano,
            delay(),
            "{}",
            span.name
        );
        assert!(span.status.is_none(), "{}", span.name);
    }

    let n_logs = model.events.len() * 2;
    let logs = semconv_otlp_logs_with_rng(n_logs, &model, &mut r);
    let records = &logs.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(records.len(), n_logs);
    for rec in records {
        assert_eq!(rec.severity_number, SeverityNumber::Info as i32);
        assert!(rec.severity_text.is_empty());
    }

    let n_metrics = model.metrics.len() * 2;
    let data = semconv_otlp_metrics_with_rng(n_metrics, &model, &mut r);
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), n_metrics);
    let mut saw_histogram = false;
    let mut daemon_values = HashSet::new();
    for metric in metrics {
        let number_points = match metric.data.as_ref().expect("data") {
            Data::Sum(s) => {
                assert_eq!(
                    s.aggregation_temporality,
                    AggregationTemporality::Unspecified as i32
                );
                &s.data_points
            }
            Data::Gauge(g) => &g.data_points,
            Data::Histogram(h) => {
                saw_histogram = true;
                assert_eq!(
                    h.aggregation_temporality,
                    AggregationTemporality::Unspecified as i32
                );
                for p in &h.data_points {
                    assert!(p.bucket_counts.is_empty(), "{}", metric.name);
                    assert!(p.explicit_bounds.is_empty(), "{}", metric.name);
                    assert_eq!(p.count, 0, "{}", metric.name);
                    assert!(p.sum.is_none() && p.min.is_none() && p.max.is_none());
                    assert_eq!(p.start_time_unix_nano, 0);
                }
                continue;
            }
            other => panic!("{}: unexpected data {other:?}", metric.name),
        };
        for p in number_points {
            assert_eq!(number_value(p.value.as_ref(), &metric.name), 1.0);
            assert_eq!(p.start_time_unix_nano, 0);
        }
        if metric.name == REPRESENTATIVE_UPDOWNCOUNTER {
            for p in number_points {
                let _ = daemon_values.insert(format!(
                    "{:?}",
                    attr_scalar(&p.attributes, "jvm.thread.daemon")
                ));
            }
        }
    }
    assert!(saw_histogram, "vendored registry has a histogram");
    // `jvm.thread.daemon` is annotated in the registry; disabled, it uses the
    // boolean type default only.
    assert_eq!(
        daemon_values,
        HashSet::from([format!("{:?}", Scalar::Bool(true))])
    );
}

// ===========================================================================
// Compiled model and samplers
// ===========================================================================

/// Scenario: A precompiled `ValueSampler` and a `ValueSampler` rebuilt for
/// every draw are driven by identically seeded RNGs for discrete, weighted,
/// and every continuous distribution (int and double output).
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
                ValueSampler::new(&spec, integer).sample_value(&mut b),
                "{spec:?} integer={integer}"
            );
        }
    }
}

/// Scenario: The vendored registry is compiled into a `SemconvModel`.
/// Guarantees: The model has one template per span group, per metric group
/// with an instrument, and per event group (in registry order), and one
/// attribute generator per group attribute, so generation cycles over exactly
/// the groups that can produce a signal.
#[test]
fn compiled_model_mirrors_registry_groups() {
    let m = model();
    let spans = groups_of(GroupType::Span);
    let metrics: Vec<_> = groups_of(GroupType::Metric)
        .into_iter()
        .filter(|g| g.instrument.is_some())
        .collect();
    let events = groups_of(GroupType::Event);
    assert_eq!(m.spans.len(), spans.len());
    assert_eq!(m.metrics.len(), metrics.len());
    assert_eq!(m.events.len(), events.len());
    for (t, g) in m.spans.iter().zip(&spans) {
        assert_eq!(t.name, g.id);
        assert_eq!(t.attributes.len(), g.attributes.len());
    }
    for (t, g) in m.metrics.iter().zip(&metrics) {
        assert_eq!(Some(&t.name), g.metric_name.as_ref());
        assert_eq!(Some(&t.instrument), g.instrument.as_ref());
        assert_eq!(t.attributes.len(), g.attributes.len());
    }
    for (t, g) in m.events.iter().zip(&events) {
        assert_eq!(Some(&t.name), g.name.as_ref());
        assert_eq!(t.attributes.len(), g.attributes.len());
    }
}

/// Scenario: A registry with an invalid event-group annotation and an invalid
/// attribute annotation on the same group is compiled.
/// Guarantees: `SemconvModel::compile` reports both errors, the group error
/// first (without an attribute) and then the attribute error, so the startup
/// error report is complete and stably ordered.
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
        YamlValue(yaml_from_str("{ severity: { values: [LOUD] } }")),
    );
    event.annotations = Some(ann);
    event.attributes.push(attribute(
        "bad.attr",
        primitive(PrimitiveOrArrayTypeSpec::Int),
        Some("{ weights: [1] }"),
    ));
    let event_id = event.id.clone();

    let compiled = SemconvModel::compile(&bad).expect_err("invalid annotations");
    assert_eq!(compiled.len(), 2);
    assert_eq!(compiled[0].group, event_id);
    assert!(compiled[0].attribute.is_none());
    assert_eq!(compiled[1].attribute.as_deref(), Some("bad.attr"));
}

/// Scenario: Non-annotated attributes are compiled into fallback generators
/// for enums, example lists, single examples, and type defaults.
/// Guarantees: Compiled fallbacks produce exactly the values the
/// non-annotated rules allow: the first enum member (as on main), every
/// example, or the type default.
#[test]
fn compiled_fallbacks_cover_expected_values() {
    let mut r = rng();
    let collect = |attr: &Attribute, r: &mut StdRng| -> HashSet<String> {
        let g = AttributeGen::try_new(attr).expect("unannotated attribute compiles");
        (0..500)
            .map(|_| format!("{:?}", kv_scalar(&g.sample(r))))
            .collect()
    };
    let e = attribute("e", enum_type(&["a", "b", "c"]), None);
    assert_eq!(
        collect(&e, &mut r),
        HashSet::from([format!("{:?}", Scalar::String("a".into()))])
    );

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

// ===========================================================================
// Limits: one data point per metric, count caps, numeric safety
// ===========================================================================

/// Scenario: Metrics are generated from every metric group in the vendored
/// registry (gauges, sums, and histograms), cycling three times.
/// Guarantees: The batch holds exactly the requested number of metrics and
/// every metric carries exactly one data point, so payload item counts
/// (data points) equal the metric count the producer requested.
#[test]
fn every_metric_carries_one_data_point() {
    let requested = model().metrics.len() * 3;
    let data = semconv_otlp_metrics_with_rng(requested, model(), &mut rng());
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), requested);
    for metric in metrics {
        assert_eq!(point_count(metric), 1, "{}", metric.name);
    }
}

/// Scenario: Metrics are requested from a model without metric templates.
/// Guarantees: Generation returns an empty batch instead of looping forever.
#[test]
fn metric_generation_without_metrics_is_empty() {
    let data = semconv_otlp_metrics_with_rng(10, &SemconvModel::default(), &mut rng());
    assert!(data.resource_metrics[0].scope_metrics[0].metrics.is_empty());
}

/// Scenario: Count specs (`observations`, array `length`) are annotated
/// with values above their caps, in both discrete and continuous form.
/// Guarantees: Each is rejected at parse time with the offending key path, so
/// a registry cannot request allocations or sampling work large enough to
/// panic or stall the receiver.
#[test]
fn parse_rejects_counts_above_caps() {
    let ns = annotation::NAMESPACE;
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { values: [1, 1001] }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.observations.values[1]"),
        "must be in 1..=1000",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { range: { min: 1, max: 1000000000000000000 } }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.observations.range.max"),
        "must be within",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { range: { min: 1, max: 1001 } }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.observations.range.max"),
        "must be in 1..=1000",
    );
    expect_err(
        parse_attr(
            "length: { values: [1025] }\nitems: { values: [1] }",
            &primitive(PrimitiveOrArrayTypeSpec::Ints),
        ),
        &format!("{ns}.length.values[0]"),
        "must be in 0..=1024",
    );
    let _ = annotation::parse_metric_spec(
        &yaml_map(&format!(
            "observations: {{ range: {{ min: 1, max: {} }} }}",
            annotation::MAX_OBSERVATIONS
        )),
        &InstrumentSpec::Histogram,
    )
    .expect("counts at the caps are valid");
    let _ = parse_attr(
        "length: { values: [0, 1024] }\nitems: { values: [1] }",
        &primitive(PrimitiveOrArrayTypeSpec::Ints),
    )
    .expect("lengths at the caps are valid");
}

/// Scenario: Int ranges are annotated with bounds beyond +/-(2^52 - 1): one
/// past the limit (+/-2^52), 2^53 + 1 on each sign (which rounds to +/-2^53
/// when converted to f64), a degenerate min == max == 2^53 + 1, and a zipf
/// range spanning nearly the whole i64 domain.
/// Guarantees: They are rejected at parse time -- validating the converted
/// bound is sound because f64 rounding never moves an out-of-limit value back
/// into range -- so int sampling never loses precision, never emits a value
/// outside the configured range, and zipf never computes an overflowing rank
/// count; bounds at exactly +/-(2^52 - 1) are accepted, compile into a
/// sampler, and only ever yield values within the range.
#[test]
fn parse_rejects_int_ranges_beyond_safe_integers() {
    let ns = annotation::NAMESPACE;
    // One past the limit (2^52) on each sign.
    expect_err(
        parse_scalar("range: { min: 0, max: 4503599627370496 }", ScalarKind::Int),
        &format!("{ns}.range.max"),
        "must be within",
    );
    expect_err(
        parse_scalar("range: { min: -4503599627370496, max: 0 }", ScalarKind::Int),
        &format!("{ns}.range.min"),
        "must be within",
    );
    // 2^53 + 1 rounds to 2^53 as an f64; validation on the converted bound
    // still rejects it because 2^53 is far above the limit.
    expect_err(
        parse_scalar("range: { min: 0, max: 9007199254740993 }", ScalarKind::Int),
        &format!("{ns}.range.max"),
        "must be within",
    );
    expect_err(
        parse_scalar("range: { min: -9007199254740993, max: 0 }", ScalarKind::Int),
        &format!("{ns}.range.min"),
        "must be within",
    );
    // Degenerate range at 2^53 + 1: rejected rather than compiling and always
    // emitting 2^53, which would be below the configured min.
    expect_err(
        parse_scalar(
            "range: { min: 9007199254740993, max: 9007199254740993 }",
            ScalarKind::Int,
        ),
        &format!("{ns}.range.min"),
        "must be within",
    );
    expect_err(
        parse_scalar(
            "range: { min: -9223372036854775808, max: 9223372036854775807 }\n\
             distribution: { type: zipf, exponent: 1.1 }",
            ScalarKind::Int,
        ),
        &format!("{ns}.range.min"),
        "must be within",
    );
    // Bounds exactly at the limit compile; zipf over the widest accepted range
    // must never produce a value outside [min, max].
    let min = -4_503_599_627_370_495.0;
    let max = 4_503_599_627_370_495.0;
    let spec = parse_scalar(
        "range: { min: -4503599627370495, max: 4503599627370495 }\n\
         distribution: { type: zipf, exponent: 1.1 }",
        ScalarKind::Int,
    )
    .expect("bounds at 2^52 - 1 are valid");
    let sampler = ValueSampler::new(&spec, true);
    let mut r = rng();
    for _ in 0..100 {
        let x = sampler.sample_f64(&mut r);
        assert!((min..=max).contains(&x), "{x}");
    }
}

/// Scenario: Weights are individually finite but their sum overflows to
/// infinity.
/// Guarantees: The spec is rejected at parse time instead of producing an
/// infinite cumulative total that would select arbitrary values.
#[test]
fn parse_rejects_weights_with_overflowing_sum() {
    expect_err(
        parse_scalar(
            "values: [1, 2, 3]\nweights: [1.0e308, 1.0e308, 0]",
            ScalarKind::Int,
        ),
        &format!("{}.weights", annotation::NAMESPACE),
        "sum must be finite",
    );
}

/// An RNG whose every draw is the maximum value, which makes `random::<f64>()`
/// return the largest value below 1.0.
struct MaxRng;

impl rand::TryRng for MaxRng {
    type Error = std::convert::Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(u32::MAX)
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(u64::MAX)
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        dst.fill(u8::MAX);
        Ok(())
    }
}

/// Scenario: A weighted sampler whose trailing items have zero weight is
/// drawn with the largest possible uniform value. With a subnormal total,
/// `u * total` rounds up to `total`, so the target lands on or past every
/// cumulative weight.
/// Guarantees: The last positive-weight index is returned; zero-weight items
/// are never selected, even when rounding pushes the target to the total.
#[test]
fn index_sampler_never_selects_trailing_zero_weights() {
    let sampler = IndexSampler::new(4, Some(&[0.1, 0.2, 0.0, 0.0]));
    assert_eq!(sampler.sample(&mut MaxRng), 1);
    let smallest_subnormal = f64::from_bits(1);
    let sampler = IndexSampler::new(3, Some(&[0.0, smallest_subnormal, 0.0]));
    assert_eq!(sampler.sample(&mut MaxRng), 1);
}

/// Scenario: A span is annotated with a duration so large that the start
/// timestamp plus the duration in nanoseconds overflows `u64`.
/// Guarantees: Generation does not panic and the end timestamp saturates at
/// `u64::MAX`, never wrapping before the start time.
#[test]
fn huge_span_duration_saturates_end_time() {
    let mut model = representative_model(&[REPRESENTATIVE_SPAN], &[], &[]);
    let spec = parse_scalar("values: [1.0e300]", ScalarKind::Double).expect("valid");
    model.spans[0].duration_ms = Some(ValueSampler::new(&spec, false));
    let traces = semconv_otlp_traces_with_rng(1, &model, &mut rng());
    let span = &traces.resource_spans[0].scope_spans[0].spans[0];
    assert_eq!(span.end_time_unix_nano, u64::MAX);
    assert!(span.end_time_unix_nano >= span.start_time_unix_nano);
}

/// Scenario: A double range spans almost the whole finite `f64` domain, so
/// its width `max - min` overflows to infinity.
/// Guarantees: Uniform samples stay finite and inside the range and are
/// spread across it, instead of every sample collapsing to `max`.
#[test]
fn uniform_sampling_handles_ranges_wider_than_f64_max() {
    let sampler = number_sampler((-1.0e308, 1.0e308), Distribution::Uniform, false);
    let xs = draw(&sampler, 1_000);
    assert!(
        xs.iter()
            .all(|x| x.is_finite() && (-1.0e308..=1.0e308).contains(x)),
        "samples must be finite and in range"
    );
    assert!(xs.iter().any(|x| *x < 0.0), "negative half is reachable");
    assert!(xs.iter().any(|x| *x > 0.0), "positive half is reachable");
    assert!(
        xs.iter().all(|x| *x < 1.0e308),
        "samples must not pin to max"
    );
}
