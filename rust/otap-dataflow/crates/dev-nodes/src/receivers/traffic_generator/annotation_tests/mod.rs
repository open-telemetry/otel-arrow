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
use super::fake_data::HISTOGRAM_BOUNDS;
use super::sampler::{
    IndexSampler, NumberSampler, ValueSampler, ZIPF_TABLE_CAP, lognormal_params, normal_cdf,
    normal_inverse_sf, normal_quantile, normal_sf,
};
use super::semconv_model::SemconvModel;
use super::semconv_signal::{
    bucket_index, semconv_otlp_logs_with_rng, semconv_otlp_metrics_with_rng,
    semconv_otlp_traces_with_rng,
};

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
    annotation::parse_attribute_spec(&yaml_map(src), ty, None)
}

fn parse_attr_with_examples(
    src: &str,
    ty: &AttributeType,
    examples: &Examples,
) -> Result<AttributeSpec, AnnotationError> {
    annotation::parse_attribute_spec(&yaml_map(src), ty, Some(examples))
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

mod generation;
mod parse;
mod sampling;
