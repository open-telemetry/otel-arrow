// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parsing and validation of `traffic-generator` annotations.
//!
//! Registry authors attach a `traffic-generator` annotation to attributes and
//! signal groups (metrics, spans, events) to describe the values the traffic
//! generator should produce. The format is documented in the "Annotations"
//! section of the receiver's `README.md`.
//!
//! The annotation has no version or type tag. The parser interprets each map
//! from its context (the attribute type or signal type, already known from the
//! resolved registry) and from which keys are present:
//!
//! - `values` (+ optional `weights`) selects the discrete form.
//! - `range` (+ optional `distribution`) selects the continuous form.
//!
//! Every map has a fixed set of allowed keys for its context; any other key is
//! rejected so typos do not silently fall back to uniform sampling.

use std::collections::BTreeMap;
use std::fmt;

use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::SeverityNumber;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::AggregationTemporality;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::status::StatusCode;
use serde_json::{Map, Value};
use weaver_semconv::YamlValue;
use weaver_semconv::v1::attribute::{
    AttributeType, EnumEntriesSpec, PrimitiveOrArrayTypeSpec, TemplateTypeSpec,
    ValueSpec as SemconvValueSpec,
};
use weaver_semconv::v1::group::InstrumentSpec;

/// A JSON object, used as the annotation mapping type.
///
/// `weaver_semconv::YamlValue` wraps a [`serde_json::Value`], so annotation
/// mappings are `serde_json` objects even though registry authors write them
/// as YAML.
type Mapping = Map<String, Value>;

/// The annotation namespace read by the traffic generator.
pub const NAMESPACE: &str = "traffic-generator";

/// Error produced while parsing or validating an annotation.
///
/// `path` is the full key path inside the annotation, starting with the
/// namespace (for example `traffic-generator.distribution.stddev`).
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationError {
    /// Key path of the offending entry.
    pub path: String,
    /// Human readable description of the problem.
    pub message: String,
}

impl AnnotationError {
    pub(crate) fn new(path: &str, message: impl Into<String>) -> Self {
        Self {
            path: path.to_owned(),
            message: message.into(),
        }
    }
}

impl fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for AnnotationError {}

/// Result alias for annotation parsing.
pub type AnnotationResult<T> = Result<T, AnnotationError>;

// The value-spec vocabulary (`ScalarKind`, `Scalar`, `Range`, `Distribution`,
// `ValueSpec`, `AttributeSpec`, `scalar_to_string`) lives in the leaf
// `value_spec` module so the parser here and the samplers can share it without
// a module cycle. Re-exported so existing `annotation::*` paths keep working.
pub use super::value_spec::{
    AttributeSpec, Distribution, Range, Scalar, ScalarKind, ValueSpec, scalar_to_string,
};

/// Signal-level spec for a metric group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetricSpec {
    /// Data point value (or per-observation value for histograms).
    pub value: Option<ValueSpec>,
    /// Number of observations aggregated into each histogram data point.
    pub observations: Option<ValueSpec>,
    /// Histogram bucket bounds (strictly increasing).
    pub bounds: Option<Vec<f64>>,
    /// Aggregation temporality for sums and histograms (only `delta`).
    pub temporality: Option<AggregationTemporality>,
}

/// Signal-level spec for a span group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpanSpec {
    /// Span duration in milliseconds.
    pub duration: Option<ValueSpec>,
    /// Span status (`unset`, `ok`, `error`).
    pub status: Option<ValueSpec>,
}

/// Signal-level spec for an event group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventSpec {
    /// Log severity (SeverityNumber short names).
    pub severity: Option<ValueSpec>,
}

/// Span `status` names and their OTLP [`StatusCode`].
///
/// Single source of truth: parsing validates names against it and
/// generation maps names through it.
pub const STATUS_TABLE: [(&str, StatusCode); 3] = [
    ("unset", StatusCode::Unset),
    ("ok", StatusCode::Ok),
    ("error", StatusCode::Error),
];

/// Event `severity` short names and their OTLP [`SeverityNumber`].
///
/// Single source of truth: parsing validates names against it and
/// generation maps names through it.
pub const SEVERITY_TABLE: [(&str, SeverityNumber); 24] = [
    ("TRACE", SeverityNumber::Trace),
    ("TRACE2", SeverityNumber::Trace2),
    ("TRACE3", SeverityNumber::Trace3),
    ("TRACE4", SeverityNumber::Trace4),
    ("DEBUG", SeverityNumber::Debug),
    ("DEBUG2", SeverityNumber::Debug2),
    ("DEBUG3", SeverityNumber::Debug3),
    ("DEBUG4", SeverityNumber::Debug4),
    ("INFO", SeverityNumber::Info),
    ("INFO2", SeverityNumber::Info2),
    ("INFO3", SeverityNumber::Info3),
    ("INFO4", SeverityNumber::Info4),
    ("WARN", SeverityNumber::Warn),
    ("WARN2", SeverityNumber::Warn2),
    ("WARN3", SeverityNumber::Warn3),
    ("WARN4", SeverityNumber::Warn4),
    ("ERROR", SeverityNumber::Error),
    ("ERROR2", SeverityNumber::Error2),
    ("ERROR3", SeverityNumber::Error3),
    ("ERROR4", SeverityNumber::Error4),
    ("FATAL", SeverityNumber::Fatal),
    ("FATAL2", SeverityNumber::Fatal2),
    ("FATAL3", SeverityNumber::Fatal3),
    ("FATAL4", SeverityNumber::Fatal4),
];

/// Distribution type, parsed once so both the key check and the
/// construction below match exhaustively.
#[derive(Clone, Copy)]
enum Kind {
    Uniform,
    Normal,
    LogNormal,
    Exponential,
    Zipf,
}

/// Map a status name to its [`StatusCode`] value.
#[must_use]
pub fn status_code(name: &str) -> Option<i32> {
    STATUS_TABLE
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, code)| *code as i32)
}

/// Map a severity short name to its [`SeverityNumber`] value and static name.
#[must_use]
pub fn severity(name: &str) -> Option<(i32, &'static str)> {
    SEVERITY_TABLE
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(n, sev)| (*sev as i32, *n))
}

/// Minimum probability mass a truncated range must carry to be accepted.
///
/// Inverse transform sampling stays accurate even for ranges deep in a tail
/// (for example normal(0, 1) on `[8, 10]`, mass ~6e-16), so only ranges whose
/// mass underflows to zero are rejected as unreachable.
pub const MIN_RANGE_MASS: f64 = f64::MIN_POSITIVE;

/// Largest accepted histogram `observations` value.
///
/// Every observation is sampled on the receiver task. This bounds the work
/// per histogram data point; work per batch still scales with
/// `max_batch_size` (at most `max_batch_size * MAX_OBSERVATIONS` samples, so
/// about 50k draws per batch at the default `max_batch_size` of 1000). The
/// receiver yields between batches, so this only bounds the uninterrupted work
/// inside a single batch.
pub const MAX_OBSERVATIONS: u64 = 50;

/// Largest accepted array attribute `length` value.
///
/// Each element is sampled on the receiver task, so this bounds the per-batch
/// work contributed by a single array attribute (at most
/// `max_batch_size * MAX_ARRAY_LENGTH` samples).
pub const MAX_ARRAY_LENGTH: u64 = 32;

/// Largest accepted number of `values` (and matching `weights`) in a discrete
/// value spec. Bounds the memory retained per compiled sampler.
pub const MAX_DISCRETE_VALUES: usize = 128;

/// Largest accepted number of histogram `bounds`. Bounds the per-point bucket
/// vector and the memory retained per compiled metric.
pub const MAX_HISTOGRAM_BOUNDS: usize = 32;

/// Largest integer magnitude accepted for int `range` bounds.
///
/// Continuous int specs are sampled as `f64`.
pub const MAX_SAFE_INT: i64 = (1 << 52) - 1;

const VALUE_SPEC_KEYS: [&str; 4] = ["values", "weights", "range", "distribution"];
const ARRAY_KEYS: [&str; 2] = ["length", "items"];
const TEMPLATE_KEYS: [&str; 2] = ["keys", "value"];
const RANGE_KEYS: [&str; 2] = ["min", "max"];

// ---------------------------------------------------------------------------
// Lookup helpers
// ---------------------------------------------------------------------------

/// Return the `traffic-generator` mapping from an annotations map, if any.
///
/// Returns an error if the namespace is present but is not a mapping.
pub fn lookup(
    annotations: Option<&BTreeMap<String, YamlValue>>,
) -> AnnotationResult<Option<&Mapping>> {
    let Some(value) = annotations.and_then(|a| a.get(NAMESPACE)) else {
        return Ok(None);
    };
    match &value.0 {
        Value::Object(m) => Ok(Some(m)),
        Value::Null => Ok(None),
        _ => Err(AnnotationError::new(NAMESPACE, "must be a mapping")),
    }
}

// ---------------------------------------------------------------------------
// Registry validation
// ---------------------------------------------------------------------------

/// An annotation error located in the registry.
///
/// Produced by [`SemconvModel::compile`], which validates every
/// `traffic-generator` annotation in the registry.
///
/// [`SemconvModel::compile`]: crate::receivers::traffic_generator::semconv_model::SemconvModel::compile
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryAnnotationError {
    /// Group id the error belongs to.
    pub group: String,
    /// Attribute name, when the error is on an attribute annotation.
    pub attribute: Option<String>,
    /// The underlying annotation error.
    pub error: AnnotationError,
}

impl fmt::Display for RegistryAnnotationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.attribute {
            Some(a) => write!(f, "{} ({a}): {}", self.group, self.error),
            None => write!(f, "{}: {}", self.group, self.error),
        }
    }
}

/// Format a list of registry annotation errors for configuration error messages.
#[must_use]
pub fn format_errors(errors: &[RegistryAnnotationError]) -> String {
    let mut out = format!("{} invalid traffic-generator annotation(s):", errors.len());
    for e in errors {
        out.push_str("\n  ");
        out.push_str(&e.to_string());
    }
    out
}

// ---------------------------------------------------------------------------
// Attribute parsing
// ---------------------------------------------------------------------------

/// Scalar kind and whether the type is an array, for a primitive-or-array spec.
const fn primitive_shape(p: &PrimitiveOrArrayTypeSpec) -> (ScalarKind, bool) {
    match p {
        PrimitiveOrArrayTypeSpec::Int => (ScalarKind::Int, false),
        PrimitiveOrArrayTypeSpec::Double => (ScalarKind::Double, false),
        PrimitiveOrArrayTypeSpec::String => (ScalarKind::String, false),
        PrimitiveOrArrayTypeSpec::Boolean => (ScalarKind::Bool, false),
        PrimitiveOrArrayTypeSpec::Any => (ScalarKind::Any, false),
        PrimitiveOrArrayTypeSpec::Ints => (ScalarKind::Int, true),
        PrimitiveOrArrayTypeSpec::Doubles => (ScalarKind::Double, true),
        PrimitiveOrArrayTypeSpec::Strings => (ScalarKind::String, true),
        PrimitiveOrArrayTypeSpec::Booleans => (ScalarKind::Bool, true),
    }
}

/// Scalar kind and whether the type is an array, for a template inner spec.
const fn template_shape(t: &TemplateTypeSpec) -> (ScalarKind, bool) {
    match t {
        TemplateTypeSpec::Int => (ScalarKind::Int, false),
        TemplateTypeSpec::Double => (ScalarKind::Double, false),
        TemplateTypeSpec::String => (ScalarKind::String, false),
        TemplateTypeSpec::Boolean => (ScalarKind::Bool, false),
        TemplateTypeSpec::Any => (ScalarKind::Any, false),
        TemplateTypeSpec::Ints => (ScalarKind::Int, true),
        TemplateTypeSpec::Doubles => (ScalarKind::Double, true),
        TemplateTypeSpec::Strings => (ScalarKind::String, true),
        TemplateTypeSpec::Booleans => (ScalarKind::Bool, true),
    }
}

/// Parse a scalar or array value spec for a `(kind, is_array)` shape.
fn scalar_or_array(
    map: &Mapping,
    path: &str,
    (kind, is_array): (ScalarKind, bool),
) -> AnnotationResult<AttributeSpec> {
    if is_array {
        array(map, path, kind)
    } else {
        scalar(map, path, kind)
    }
}

/// Parse an attribute annotation for the given semconv attribute type.
pub fn parse_attribute_spec(map: &Mapping, ty: &AttributeType) -> AnnotationResult<AttributeSpec> {
    let path = NAMESPACE;
    match ty {
        AttributeType::PrimitiveOrArray(p) => scalar_or_array(map, path, primitive_shape(p)),
        AttributeType::Enum { members } => {
            let spec = parse_value_spec(map, path, enum_kind(members))?;
            check_enum_members(&spec, members, path)?;
            Ok(AttributeSpec::Scalar(spec))
        }
        AttributeType::Template(t) => {
            check_keys(map, path, &TEMPLATE_KEYS)?;
            let keys_map = required_map(map, path, "keys")?;
            let keys_path = format!("{path}.keys");
            let keys = parse_value_spec(keys_map, &keys_path, ScalarKind::String)?;
            let value_map = required_map(map, path, "value")?;
            let value = scalar_or_array(value_map, &format!("{path}.value"), template_shape(t))?;
            Ok(AttributeSpec::Template {
                keys,
                value: Box::new(value),
            })
        }
    }
}

fn scalar(map: &Mapping, path: &str, kind: ScalarKind) -> AnnotationResult<AttributeSpec> {
    if map.contains_key("items") || map.contains_key("length") {
        return Err(AnnotationError::new(
            path,
            "`length` and `items` are only valid on array attributes",
        ));
    }
    if map.contains_key("keys") {
        return Err(AnnotationError::new(
            path,
            "`keys` is only valid on template attributes",
        ));
    }
    Ok(AttributeSpec::Scalar(parse_value_spec(map, path, kind)?))
}

fn array(map: &Mapping, path: &str, kind: ScalarKind) -> AnnotationResult<AttributeSpec> {
    if map.contains_key("values") || map.contains_key("range") {
        return Err(AnnotationError::new(
            path,
            "array attributes require `length` and `items` instead of `values` or `range`",
        ));
    }
    check_keys(map, path, &ARRAY_KEYS)?;
    let length_map = required_map(map, path, "length")?;
    let length_path = format!("{path}.length");
    let length = parse_value_spec(length_map, &length_path, ScalarKind::Int)?;
    check_count(&length, &length_path, 0, MAX_ARRAY_LENGTH)?;
    let items_map = required_map(map, path, "items")?;
    let items = parse_value_spec(items_map, &format!("{path}.items"), kind)?;
    Ok(AttributeSpec::Array { length, items })
}

/// Scalar kind used to type-check an enum annotation's `values`.
///
/// Enums only accept the discrete form, and membership is checked separately,
/// so the kind only needs to be strict enough to catch obvious mismatches:
/// all-string enums expect strings and all-bool enums expect bools; everything
/// else (numeric or mixed) uses `Any`, which accepts any scalar.
fn enum_kind(members: &[EnumEntriesSpec]) -> ScalarKind {
    let all = |pred: fn(&SemconvValueSpec) -> bool| {
        !members.is_empty() && members.iter().all(|m| pred(&m.value))
    };
    if all(|v| matches!(v, SemconvValueSpec::String(_))) {
        ScalarKind::String
    } else if all(|v| matches!(v, SemconvValueSpec::Bool(_))) {
        ScalarKind::Bool
    } else {
        ScalarKind::Any
    }
}

fn check_enum_members(
    spec: &ValueSpec,
    members: &[EnumEntriesSpec],
    path: &str,
) -> AnnotationResult<()> {
    let ValueSpec::Discrete { values, .. } = spec else {
        return Err(AnnotationError::new(
            &format!("{path}.range"),
            "enum attributes only accept the discrete form (`values`)",
        ));
    };
    for (i, v) in values.iter().enumerate() {
        let found = members.iter().any(|m| match (&m.value, v) {
            (SemconvValueSpec::String(a), Scalar::String(b)) => a == b,
            (SemconvValueSpec::Int(a), Scalar::Int(b)) => a == b,
            (SemconvValueSpec::Double(a), Scalar::Double(b)) => f64::from(*a) == *b,
            (SemconvValueSpec::Double(a), Scalar::Int(b)) => f64::from(*a) == *b as f64,
            (SemconvValueSpec::Bool(a), Scalar::Bool(b)) => a == b,
            _ => false,
        });
        if !found {
            let shown = match v {
                Scalar::String(s) => format!("\"{s}\""),
                other => scalar_to_string(other),
            };
            return Err(AnnotationError::new(
                &format!("{path}.values[{i}]"),
                format!("{shown} is not an enum member value"),
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Signal parsing
// ---------------------------------------------------------------------------

/// Parse a metric group annotation for the given instrument.
pub fn parse_metric_spec(
    map: &Mapping,
    instrument: &InstrumentSpec,
) -> AnnotationResult<MetricSpec> {
    let path = NAMESPACE;
    let histogram = matches!(instrument, InstrumentSpec::Histogram);
    let gauge = matches!(instrument, InstrumentSpec::Gauge);
    for key in map.keys() {
        let reason = match key.as_str() {
            "value" => continue,
            "observations" | "bounds" if histogram => continue,
            "temporality" if !gauge => continue,
            "observations" | "bounds" => {
                format!("unknown key \"{key}\" for non-histogram metric (only valid on histograms)")
            }
            "temporality" => {
                format!("unknown key \"{key}\" for gauge (only valid on sums and histograms)")
            }
            _ => format!("unknown key \"{key}\" for metric group"),
        };
        return Err(AnnotationError::new(path, reason));
    }
    let mut spec = MetricSpec::default();
    if let Some(m) = optional_map(map, path, "value")? {
        let p = format!("{path}.value");
        let v = parse_value_spec(m, &p, ScalarKind::Double)?;
        // Counter increments and histogram observations must be >= 0 (OTLP
        // only defines a histogram `sum` for non-negative observations).
        if matches!(instrument, InstrumentSpec::Counter) || histogram {
            check_non_negative(&v, &p)?;
        }
        spec.value = Some(v);
    }
    if let Some(t) = map.get("temporality") {
        spec.temporality = Some(parse_temporality(t, &format!("{path}.temporality"))?);
    }
    if let Some(m) = optional_map(map, path, "observations")? {
        let p = format!("{path}.observations");
        let v = parse_value_spec(m, &p, ScalarKind::Int)?;
        check_count(&v, &p, 1, MAX_OBSERVATIONS)?;
        spec.observations = Some(v);
    }
    if let Some(b) = map.get("bounds") {
        spec.bounds = Some(parse_bounds(b, &format!("{path}.bounds"))?);
    }
    Ok(spec)
}

/// Parse a span group annotation.
pub fn parse_span_spec(map: &Mapping) -> AnnotationResult<SpanSpec> {
    let path = NAMESPACE;
    for key in map.keys() {
        if !["duration", "status"].contains(&key.as_str()) {
            return Err(AnnotationError::new(
                path,
                format!("unknown key \"{key}\" for span group"),
            ));
        }
    }
    let mut spec = SpanSpec::default();
    if let Some(m) = optional_map(map, path, "duration")? {
        let p = format!("{path}.duration");
        let v = parse_value_spec(m, &p, ScalarKind::Double)?;
        check_non_negative(&v, &p)?;
        spec.duration = Some(v);
    }
    if let Some(m) = optional_map(map, path, "status")? {
        let p = format!("{path}.status");
        let v = parse_value_spec(m, &p, ScalarKind::String)?;
        check_names(&v, &p, |s| status_code(s).is_some(), "status")?;
        spec.status = Some(v);
    }
    Ok(spec)
}

/// Parse an event group annotation.
pub fn parse_event_spec(map: &Mapping) -> AnnotationResult<EventSpec> {
    let path = NAMESPACE;
    for key in map.keys() {
        if key != "severity" {
            return Err(AnnotationError::new(
                path,
                format!("unknown key \"{key}\" for event group"),
            ));
        }
    }
    let mut spec = EventSpec::default();
    if let Some(m) = optional_map(map, path, "severity")? {
        let p = format!("{path}.severity");
        let v = parse_value_spec(m, &p, ScalarKind::String)?;
        check_names(&v, &p, |s| severity(s).is_some(), "severity")?;
        spec.severity = Some(v);
    }
    Ok(spec)
}

/// Parse a metric `temporality`.
///
/// Only `delta` is supported: every generated point is an independent sample,
/// so cumulative totals would decrease between points (read as resets) and
/// repeat on `pre_generated` replay.
fn parse_temporality(value: &Value, path: &str) -> AnnotationResult<AggregationTemporality> {
    match value {
        Value::String(s) if s == "delta" => Ok(AggregationTemporality::Delta),
        Value::String(s) => Err(AnnotationError::new(
            path,
            format!("unsupported temporality \"{s}\" (expected delta)"),
        )),
        _ => Err(AnnotationError::new(path, "must be a string")),
    }
}

fn parse_bounds(value: &Value, path: &str) -> AnnotationResult<Vec<f64>> {
    let Value::Array(seq) = value else {
        return Err(AnnotationError::new(path, "must be a list of numbers"));
    };
    if seq.is_empty() {
        return Err(AnnotationError::new(path, "must not be empty"));
    }
    if seq.len() > MAX_HISTOGRAM_BOUNDS {
        return Err(AnnotationError::new(
            path,
            format!("must have at most {MAX_HISTOGRAM_BOUNDS} bounds"),
        ));
    }
    let mut out = Vec::with_capacity(seq.len());
    for (i, v) in seq.iter().enumerate() {
        let n = number(v, &format!("{path}[{i}]"))?;
        if let Some(prev) = out.last()
            && n <= *prev
        {
            return Err(AnnotationError::new(
                &format!("{path}[{i}]"),
                "bounds must be strictly increasing",
            ));
        }
        out.push(n);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Value spec parsing
// ---------------------------------------------------------------------------

/// Parse a value spec (discrete or continuous) producing the given scalar kind.
pub fn parse_value_spec(
    map: &Mapping,
    path: &str,
    kind: ScalarKind,
) -> AnnotationResult<ValueSpec> {
    check_keys(map, path, &VALUE_SPEC_KEYS)?;
    let has_values = map.contains_key("values");
    let has_range = map.contains_key("range");
    let has_weights = map.contains_key("weights");
    let has_distribution = map.contains_key("distribution");

    match (has_values, has_range) {
        (true, true) => {
            return Err(AnnotationError::new(
                path,
                "`values` and `range` are mutually exclusive",
            ));
        }
        (false, false) => {
            return Err(AnnotationError::new(
                path,
                "a value spec requires `values` or `range`",
            ));
        }
        _ => {}
    }
    if has_weights && !has_values {
        return Err(AnnotationError::new(path, "`weights` requires `values`"));
    }
    if has_distribution && !has_range {
        return Err(AnnotationError::new(
            path,
            "`distribution` requires `range`",
        ));
    }

    if has_values {
        let values_path = format!("{path}.values");
        let Some(Value::Array(seq)) = map.get("values") else {
            return Err(AnnotationError::new(&values_path, "must be a list"));
        };
        if seq.is_empty() {
            return Err(AnnotationError::new(&values_path, "must not be empty"));
        }
        if seq.len() > MAX_DISCRETE_VALUES {
            return Err(AnnotationError::new(
                &values_path,
                format!("must have at most {MAX_DISCRETE_VALUES} values"),
            ));
        }
        let values = seq
            .iter()
            .enumerate()
            .map(|(i, v)| scalar_value(v, &format!("{values_path}[{i}]"), kind))
            .collect::<AnnotationResult<Vec<_>>>()?;
        let weights = match map.get("weights") {
            None => None,
            Some(w) => Some(parse_weights(w, &format!("{path}.weights"), values.len())?),
        };
        Ok(ValueSpec::Discrete { values, weights })
    } else {
        if !kind.is_numeric() {
            return Err(AnnotationError::new(
                &format!("{path}.range"),
                format!("continuous form is not allowed for {}", kind.name()),
            ));
        }
        let range = parse_range(map.get("range"), &format!("{path}.range"), kind)?;
        let distribution = match map.get("distribution") {
            None => Distribution::Uniform,
            Some(d) => parse_distribution(d, &format!("{path}.distribution"), kind)?,
        };
        validate_distribution(&range, &distribution, path, kind)?;
        Ok(ValueSpec::Continuous {
            range,
            distribution,
        })
    }
}

fn parse_weights(value: &Value, path: &str, len: usize) -> AnnotationResult<Vec<f64>> {
    let Value::Array(seq) = value else {
        return Err(AnnotationError::new(path, "must be a list of numbers"));
    };
    if seq.len() != len {
        return Err(AnnotationError::new(
            path,
            format!("length {} does not match `values` length {len}", seq.len()),
        ));
    }
    let mut out = Vec::with_capacity(seq.len());
    for (i, v) in seq.iter().enumerate() {
        let p = format!("{path}[{i}]");
        let w = number(v, &p)?;
        if w < 0.0 {
            return Err(AnnotationError::new(&p, "weights must be >= 0"));
        }
        out.push(w);
    }
    let total = out.iter().sum::<f64>();
    if !total.is_finite() {
        return Err(AnnotationError::new(path, "weights sum must be finite"));
    }
    if total <= 0.0 {
        return Err(AnnotationError::new(path, "weights must sum to > 0"));
    }
    Ok(out)
}

/// Parse and validate a single `range` bound.
///
/// For int specs the bound must be a whole number within +/-[`MAX_SAFE_INT`].
/// Both checks run on the converted `f64`: an `f64` has a 53-bit mantissa, so
/// every integer up to `MAX_SAFE_INT` (2^52 - 1) is represented exactly and the
/// whole-number and range checks are precise at that scale.
fn range_bound(m: &Mapping, key: &str, path: &str, kind: ScalarKind) -> AnnotationResult<f64> {
    let p = format!("{path}.{key}");
    let v = number(
        m.get(key)
            .ok_or_else(|| AnnotationError::new(path, format!("missing `{key}`")))?,
        &p,
    )?;
    if kind == ScalarKind::Int {
        if v.fract() != 0.0 {
            return Err(AnnotationError::new(
                &p,
                "int ranges require integer bounds",
            ));
        }
        let limit = MAX_SAFE_INT as f64;
        if !(-limit..=limit).contains(&v) {
            return Err(AnnotationError::new(
                &p,
                format!("int range bounds must be within +/-{MAX_SAFE_INT}"),
            ));
        }
    }
    Ok(v)
}

fn parse_range(value: Option<&Value>, path: &str, kind: ScalarKind) -> AnnotationResult<Range> {
    let Some(Value::Object(m)) = value else {
        return Err(AnnotationError::new(
            path,
            "must be a mapping with `min` and `max`",
        ));
    };
    check_keys(m, path, &RANGE_KEYS)?;
    let min = range_bound(m, "min", path, kind)?;
    let max = range_bound(m, "max", path, kind)?;
    if min > max {
        return Err(AnnotationError::new(
            path,
            format!("min ({min}) must be <= max ({max})"),
        ));
    }
    Ok(Range { min, max })
}

fn parse_distribution(
    value: &Value,
    path: &str,
    kind: ScalarKind,
) -> AnnotationResult<Distribution> {
    let Value::Object(m) = value else {
        return Err(AnnotationError::new(path, "must be a mapping with `type`"));
    };
    let ty = m
        .get("type")
        .ok_or_else(|| AnnotationError::new(path, "missing `type`"))?;
    let ty = ty
        .as_str()
        .ok_or_else(|| AnnotationError::new(&format!("{path}.type"), "must be a string"))?;

    let (dist_kind, allowed): (Kind, &[&str]) = match ty {
        "uniform" => (Kind::Uniform, &[]),
        "normal" => (Kind::Normal, &["mean", "stddev"]),
        "lognormal" => (Kind::LogNormal, &["mean", "stddev"]),
        "exponential" => (Kind::Exponential, &["mean"]),
        "zipf" => (Kind::Zipf, &["exponent"]),
        other => {
            return Err(AnnotationError::new(
                &format!("{path}.type"),
                format!(
                    "unknown distribution \"{other}\" (expected uniform, normal, lognormal, exponential, zipf)"
                ),
            ));
        }
    };
    for key in m.keys() {
        if key != "type" && !allowed.contains(&key.as_str()) {
            return Err(AnnotationError::new(
                path,
                format!("unknown key \"{key}\" for type {ty}"),
            ));
        }
    }

    let param = |name: &str| -> AnnotationResult<f64> {
        let v = m.get(name).ok_or_else(|| {
            AnnotationError::new(path, format!("missing required `{name}` for type {ty}"))
        })?;
        number(v, &format!("{path}.{name}"))
    };
    let positive = |name: &str| -> AnnotationResult<f64> {
        let v = param(name)?;
        if v > 0.0 {
            Ok(v)
        } else {
            Err(AnnotationError::new(
                &format!("{path}.{name}"),
                format!("must be > 0 for type {ty}"),
            ))
        }
    };

    match dist_kind {
        Kind::Uniform => Ok(Distribution::Uniform),
        Kind::Normal => Ok(Distribution::Normal {
            mean: param("mean")?,
            stddev: positive("stddev")?,
        }),
        Kind::LogNormal => Ok(Distribution::LogNormal {
            mean: positive("mean")?,
            stddev: positive("stddev")?,
        }),
        Kind::Exponential => Ok(Distribution::Exponential {
            mean: positive("mean")?,
        }),
        Kind::Zipf if kind != ScalarKind::Int => Err(AnnotationError::new(
            &format!("{path}.type"),
            "zipf only applies to int values",
        )),
        Kind::Zipf => Ok(Distribution::Zipf {
            exponent: positive("exponent")?,
        }),
    }
}

fn validate_distribution(
    range: &Range,
    distribution: &Distribution,
    path: &str,
    kind: ScalarKind,
) -> AnnotationResult<()> {
    let dpath = format!("{path}.distribution");
    match distribution {
        Distribution::LogNormal { .. } if range.max <= 0.0 => Err(AnnotationError::new(
            &dpath,
            "lognormal requires range.max > 0",
        )),
        Distribution::Exponential { .. } if range.max < 0.0 => Err(AnnotationError::new(
            &dpath,
            "exponential requires range.max >= 0",
        )),
        _ => {
            let mass = crate::receivers::traffic_generator::sampler::range_mass(
                range,
                distribution,
                kind == ScalarKind::Int,
            );
            if mass.is_finite() && mass >= MIN_RANGE_MASS {
                Ok(())
            } else {
                Err(AnnotationError::new(
                    &dpath,
                    format!(
                        "range [{}, {}] has ~0 probability under {}",
                        range.min,
                        range.max,
                        distribution_display(distribution)
                    ),
                ))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

fn check_keys(map: &Mapping, path: &str, allowed: &[&str]) -> AnnotationResult<()> {
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(AnnotationError::new(path, format!("unknown key \"{key}\"")));
        }
    }
    Ok(())
}

fn required_map<'a>(map: &'a Mapping, path: &str, key: &str) -> AnnotationResult<&'a Mapping> {
    match map.get(key) {
        Some(Value::Object(m)) => Ok(m),
        Some(_) => Err(AnnotationError::new(
            &format!("{path}.{key}"),
            "must be a mapping",
        )),
        None => Err(AnnotationError::new(
            path,
            format!("missing required `{key}`"),
        )),
    }
}

fn optional_map<'a>(
    map: &'a Mapping,
    path: &str,
    key: &str,
) -> AnnotationResult<Option<&'a Mapping>> {
    match map.get(key) {
        Some(Value::Object(m)) => Ok(Some(m)),
        Some(_) => Err(AnnotationError::new(
            &format!("{path}.{key}"),
            "must be a mapping",
        )),
        None => Ok(None),
    }
}

fn number(value: &Value, path: &str) -> AnnotationResult<f64> {
    let n = match value {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| AnnotationError::new(path, "must be a number"))?,
        _ => return Err(AnnotationError::new(path, "must be a number")),
    };
    if n.is_finite() {
        Ok(n)
    } else {
        Err(AnnotationError::new(path, "must be finite"))
    }
}

fn scalar_value(value: &Value, path: &str, kind: ScalarKind) -> AnnotationResult<Scalar> {
    let mismatch = || {
        AnnotationError::new(
            path,
            format!("value type does not match attribute type {}", kind.name()),
        )
    };
    match (kind, value) {
        (ScalarKind::Int, Value::Number(n)) => n.as_i64().map(Scalar::Int).ok_or_else(mismatch),
        (ScalarKind::Double, Value::Number(n)) => {
            let f = n.as_f64().ok_or_else(mismatch)?;
            if f.is_finite() {
                Ok(Scalar::Double(f))
            } else {
                Err(AnnotationError::new(path, "must be finite"))
            }
        }
        (ScalarKind::String, Value::String(s)) => Ok(Scalar::String(s.clone())),
        (ScalarKind::Bool, Value::Bool(b)) => Ok(Scalar::Bool(*b)),
        (ScalarKind::Any, Value::Number(n)) => match n.as_i64() {
            Some(i) => Ok(Scalar::Int(i)),
            None => n.as_f64().map(Scalar::Double).ok_or_else(mismatch),
        },
        (ScalarKind::Any, Value::String(s)) => Ok(Scalar::String(s.clone())),
        (ScalarKind::Any, Value::Bool(b)) => Ok(Scalar::Bool(*b)),
        _ => Err(mismatch()),
    }
}

fn check_non_negative(spec: &ValueSpec, path: &str) -> AnnotationResult<()> {
    match spec {
        ValueSpec::Continuous { range, .. } if range.min < 0.0 => Err(AnnotationError::new(
            &format!("{path}.range.min"),
            "must be >= 0",
        )),
        ValueSpec::Discrete { values, .. } => {
            for (i, v) in values.iter().enumerate() {
                let n = match v {
                    Scalar::Int(i) => *i as f64,
                    Scalar::Double(d) => *d,
                    _ => continue,
                };
                if n < 0.0 {
                    return Err(AnnotationError::new(
                        &format!("{path}.values[{i}]"),
                        "must be >= 0",
                    ));
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Check that an int count spec only produces values in `min..=max`.
fn check_count(spec: &ValueSpec, path: &str, min: u64, max: u64) -> AnnotationResult<()> {
    let message = || format!("must be in {min}..={max}");
    let in_bounds = |n: f64| n >= min as f64 && n <= max as f64;
    match spec {
        ValueSpec::Continuous { range, .. } => {
            if !in_bounds(range.min) {
                return Err(AnnotationError::new(
                    &format!("{path}.range.min"),
                    message(),
                ));
            }
            if !in_bounds(range.max) {
                return Err(AnnotationError::new(
                    &format!("{path}.range.max"),
                    message(),
                ));
            }
            Ok(())
        }
        ValueSpec::Discrete { values, .. } => {
            for (i, v) in values.iter().enumerate() {
                if let Scalar::Int(n) = v
                    && !in_bounds(*n as f64)
                {
                    return Err(AnnotationError::new(
                        &format!("{path}.values[{i}]"),
                        message(),
                    ));
                }
            }
            Ok(())
        }
    }
}

/// Check that every value of a discrete string spec is a known name.
fn check_names(
    spec: &ValueSpec,
    path: &str,
    is_known: impl Fn(&str) -> bool,
    what: &str,
) -> AnnotationResult<()> {
    if let ValueSpec::Discrete { values, .. } = spec {
        for (i, v) in values.iter().enumerate() {
            if let Scalar::String(s) = v
                && !is_known(s)
            {
                return Err(AnnotationError::new(
                    &format!("{path}.values[{i}]"),
                    format!("unknown {what} \"{s}\""),
                ));
            }
        }
    }
    Ok(())
}

fn distribution_display(d: &Distribution) -> String {
    match d {
        Distribution::Uniform => "uniform".to_owned(),
        Distribution::Normal { mean, stddev } => format!("normal(mean={mean}, stddev={stddev})"),
        Distribution::LogNormal { mean, stddev } => {
            format!("lognormal(mean={mean}, stddev={stddev})")
        }
        Distribution::Exponential { mean } => format!("exponential(mean={mean})"),
        Distribution::Zipf { exponent } => format!("zipf(exponent={exponent})"),
    }
}
