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

/// The kind of scalar a value spec produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    /// 64-bit signed integer.
    Int,
    /// 64-bit float.
    Double,
    /// UTF-8 string.
    String,
    /// Boolean.
    Bool,
    /// Any scalar (int, double, string, or boolean), discrete form only.
    Any,
}

impl ScalarKind {
    const fn name(self) -> &'static str {
        match self {
            ScalarKind::Int => "int",
            ScalarKind::Double => "double",
            ScalarKind::String => "string",
            ScalarKind::Bool => "boolean",
            ScalarKind::Any => "any",
        }
    }

    const fn is_numeric(self) -> bool {
        matches!(self, ScalarKind::Int | ScalarKind::Double)
    }
}

/// A scalar value in a discrete value list.
#[derive(Debug, Clone, PartialEq)]
pub enum Scalar {
    /// Integer value.
    Int(i64),
    /// Floating point value.
    Double(f64),
    /// String value.
    String(String),
    /// Boolean value.
    Bool(bool),
}

/// An inclusive numeric range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    /// Inclusive lower bound.
    pub min: f64,
    /// Inclusive upper bound.
    pub max: f64,
}

/// Distribution used to sample a continuous range.
///
/// Parameters are expressed in the units of the generated value and describe
/// the distribution before truncation to the range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distribution {
    /// Uniform over the range.
    Uniform,
    /// Normal (Gaussian).
    Normal {
        /// Mean of the distribution.
        mean: f64,
        /// Standard deviation of the distribution.
        stddev: f64,
    },
    /// Log-normal parameterized by the mean and standard deviation of the value.
    LogNormal {
        /// Mean of the value.
        mean: f64,
        /// Standard deviation of the value.
        stddev: f64,
    },
    /// Exponential parameterized by its mean (rate = 1 / mean).
    Exponential {
        /// Mean of the distribution.
        mean: f64,
    },
    /// Zipf over integer ranks; rank 1 is `range.min`.
    Zipf {
        /// Zipf exponent.
        exponent: f64,
    },
}

/// A validated value spec: exactly one of the discrete or continuous forms.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueSpec {
    /// Pick one of `values`, uniformly or by `weights`.
    Discrete {
        /// Candidate values.
        values: Vec<Scalar>,
        /// Optional relative weights, same length as `values`.
        weights: Option<Vec<f64>>,
    },
    /// Sample a number in `range` using `distribution`.
    Continuous {
        /// Inclusive range.
        range: Range,
        /// Distribution (uniform when omitted).
        distribution: Distribution,
    },
}

/// Spec for an attribute value, depending on the attribute type.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeSpec {
    /// Scalar attribute (int, double, string, boolean, enum, any).
    Scalar(ValueSpec),
    /// Array attribute: a length spec and an item spec.
    Array {
        /// Int spec for the array length.
        length: ValueSpec,
        /// Spec for each element.
        items: ValueSpec,
    },
    /// Template attribute: a key spec and a value spec.
    Template {
        /// String spec for the key appended to the template prefix.
        keys: ValueSpec,
        /// Spec for the value, according to the template's inner type.
        value: Box<AttributeSpec>,
    },
}

/// Signal-level spec for a metric group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetricSpec {
    /// Data point value (or per-observation value for histograms).
    pub value: Option<ValueSpec>,
    /// Number of observations aggregated into each histogram data point.
    pub observations: Option<ValueSpec>,
    /// Histogram bucket bounds (strictly increasing).
    pub bounds: Option<Vec<f64>>,
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

/// Allowed status names for span `status`.
pub const STATUS_NAMES: [&str; 3] = ["unset", "ok", "error"];

/// Allowed severity names for event `severity`, indexed by SeverityNumber - 1.
pub const SEVERITY_NAMES: [&str; 24] = [
    "TRACE", "TRACE2", "TRACE3", "TRACE4", "DEBUG", "DEBUG2", "DEBUG3", "DEBUG4", "INFO", "INFO2",
    "INFO3", "INFO4", "WARN", "WARN2", "WARN3", "WARN4", "ERROR", "ERROR2", "ERROR3", "ERROR4",
    "FATAL", "FATAL2", "FATAL3", "FATAL4",
];

/// OTel SDK default explicit histogram bucket boundaries.
pub const DEFAULT_HISTOGRAM_BOUNDS: [f64; 15] = [
    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0, 7500.0,
    10000.0,
];

/// Minimum probability mass a truncated range must carry to be accepted.
///
/// Inverse transform sampling stays accurate even for ranges deep in a tail
/// (for example normal(0, 1) on `[8, 10]`, mass ~6e-16), so only ranges whose
/// mass underflows to zero are rejected as unreachable.
pub const MIN_RANGE_MASS: f64 = f64::MIN_POSITIVE;

/// Largest accepted histogram `observations` value.
///
/// Every observation is sampled on the receiver task, so a histogram batch
/// does at most `max_batch_size * MAX_OBSERVATIONS` samples.
pub const MAX_OBSERVATIONS: u64 = 100_000;

/// Largest accepted array attribute `length` value.
pub const MAX_ARRAY_LENGTH: u64 = 1_024;

/// Largest integer magnitude accepted for int `range` bounds.
///
/// Continuous int specs are sampled as `f64`, which represents every integer
/// exactly only up to 2^53. Discrete int `values` are emitted as-is and are
/// not limited.
pub const MAX_SAFE_INT: i64 = 1 << 53;

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

/// Parse an attribute annotation for the given semconv attribute type.
pub fn parse_attribute_spec(map: &Mapping, ty: &AttributeType) -> AnnotationResult<AttributeSpec> {
    let path = NAMESPACE;
    match ty {
        AttributeType::PrimitiveOrArray(p) => match p {
            PrimitiveOrArrayTypeSpec::Int => scalar(map, path, ScalarKind::Int),
            PrimitiveOrArrayTypeSpec::Double => scalar(map, path, ScalarKind::Double),
            PrimitiveOrArrayTypeSpec::String => scalar(map, path, ScalarKind::String),
            PrimitiveOrArrayTypeSpec::Boolean => scalar(map, path, ScalarKind::Bool),
            PrimitiveOrArrayTypeSpec::Any => scalar(map, path, ScalarKind::Any),
            PrimitiveOrArrayTypeSpec::Ints => array(map, path, ScalarKind::Int),
            PrimitiveOrArrayTypeSpec::Doubles => array(map, path, ScalarKind::Double),
            PrimitiveOrArrayTypeSpec::Strings => array(map, path, ScalarKind::String),
            PrimitiveOrArrayTypeSpec::Booleans => array(map, path, ScalarKind::Bool),
        },
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
            let value_path = format!("{path}.value");
            let value = match t {
                TemplateTypeSpec::Int => scalar(value_map, &value_path, ScalarKind::Int)?,
                TemplateTypeSpec::Double => scalar(value_map, &value_path, ScalarKind::Double)?,
                TemplateTypeSpec::String => scalar(value_map, &value_path, ScalarKind::String)?,
                TemplateTypeSpec::Boolean => scalar(value_map, &value_path, ScalarKind::Bool)?,
                TemplateTypeSpec::Any => scalar(value_map, &value_path, ScalarKind::Any)?,
                TemplateTypeSpec::Ints => array(value_map, &value_path, ScalarKind::Int)?,
                TemplateTypeSpec::Doubles => array(value_map, &value_path, ScalarKind::Double)?,
                TemplateTypeSpec::Strings => array(value_map, &value_path, ScalarKind::String)?,
                TemplateTypeSpec::Booleans => array(value_map, &value_path, ScalarKind::Bool)?,
            };
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

fn enum_kind(members: &[EnumEntriesSpec]) -> ScalarKind {
    let mut kind = None;
    for m in members {
        let k = match m.value {
            SemconvValueSpec::Int(_) => ScalarKind::Int,
            SemconvValueSpec::Double(_) => ScalarKind::Double,
            SemconvValueSpec::String(_) => ScalarKind::String,
            SemconvValueSpec::Bool(_) => ScalarKind::Bool,
        };
        match kind {
            None => kind = Some(k),
            Some(prev) if prev != k => return ScalarKind::Any,
            _ => {}
        }
    }
    // Enums only accept the discrete form. Using Any for mixed enums keeps the
    // value check permissive; membership is checked separately.
    match kind {
        Some(ScalarKind::Int) | Some(ScalarKind::Double) => ScalarKind::Any,
        Some(k) => k,
        None => ScalarKind::Any,
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
            return Err(AnnotationError::new(
                &format!("{path}.values[{i}]"),
                format!("{} is not an enum member value", scalar_display(v)),
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
    let allowed: &[&str] = if histogram {
        &["value", "observations", "bounds"]
    } else {
        &["value"]
    };
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            let reason = if matches!(key.as_str(), "observations" | "bounds") {
                format!("unknown key \"{key}\" for non-histogram metric (only valid on histograms)")
            } else {
                format!("unknown key \"{key}\" for metric group")
            };
            return Err(AnnotationError::new(path, reason));
        }
    }
    let mut spec = MetricSpec::default();
    if let Some(m) = optional_map(map, path, "value")? {
        let p = format!("{path}.value");
        let v = parse_value_spec(m, &p, ScalarKind::Double)?;
        if matches!(instrument, InstrumentSpec::Counter) {
            check_non_negative(&v, &p)?;
        }
        spec.value = Some(v);
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
        check_names(&v, &p, &STATUS_NAMES, "status")?;
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
        check_names(&v, &p, &SEVERITY_NAMES, "severity")?;
        spec.severity = Some(v);
    }
    Ok(spec)
}

fn parse_bounds(value: &Value, path: &str) -> AnnotationResult<Vec<f64>> {
    let Value::Array(seq) = value else {
        return Err(AnnotationError::new(path, "must be a list of numbers"));
    };
    if seq.is_empty() {
        return Err(AnnotationError::new(path, "must not be empty"));
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
        let range = parse_range(map.get("range"), &format!("{path}.range"))?;
        if kind == ScalarKind::Int {
            if range.min.fract() != 0.0 || range.max.fract() != 0.0 {
                return Err(AnnotationError::new(
                    &format!("{path}.range"),
                    "int ranges require integer bounds",
                ));
            }
            let limit = MAX_SAFE_INT as f64;
            if range.min < -limit || range.max > limit {
                return Err(AnnotationError::new(
                    &format!("{path}.range"),
                    format!("int range bounds must be within +/-{MAX_SAFE_INT}"),
                ));
            }
        }
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

fn parse_range(value: Option<&Value>, path: &str) -> AnnotationResult<Range> {
    let Some(Value::Object(m)) = value else {
        return Err(AnnotationError::new(
            path,
            "must be a mapping with `min` and `max`",
        ));
    };
    check_keys(m, path, &RANGE_KEYS)?;
    let min = number(
        m.get("min")
            .ok_or_else(|| AnnotationError::new(path, "missing `min`"))?,
        &format!("{path}.min"),
    )?;
    let max = number(
        m.get("max")
            .ok_or_else(|| AnnotationError::new(path, "missing `max`"))?,
        &format!("{path}.max"),
    )?;
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
    let params: &[&str] = match ty {
        "uniform" => &[],
        "normal" | "lognormal" => &["mean", "stddev"],
        "exponential" => &["mean"],
        "zipf" => &["exponent"],
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
        if key != "type" && !params.contains(&key.as_str()) {
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
    let positive = |name: &str, v: f64| -> AnnotationResult<f64> {
        if v > 0.0 {
            Ok(v)
        } else {
            Err(AnnotationError::new(
                &format!("{path}.{name}"),
                format!("must be > 0 for type {ty}"),
            ))
        }
    };
    match ty {
        "uniform" => Ok(Distribution::Uniform),
        "normal" => Ok(Distribution::Normal {
            mean: param("mean")?,
            stddev: positive("stddev", param("stddev")?)?,
        }),
        "lognormal" => Ok(Distribution::LogNormal {
            mean: positive("mean", param("mean")?)?,
            stddev: positive("stddev", param("stddev")?)?,
        }),
        "exponential" => Ok(Distribution::Exponential {
            mean: positive("mean", param("mean")?)?,
        }),
        "zipf" => {
            if kind != ScalarKind::Int {
                return Err(AnnotationError::new(
                    &format!("{path}.type"),
                    "zipf only applies to int values",
                ));
            }
            Ok(Distribution::Zipf {
                exponent: positive("exponent", param("exponent")?)?,
            })
        }
        _ => unreachable!("distribution type validated above"),
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

fn check_names(spec: &ValueSpec, path: &str, allowed: &[&str], what: &str) -> AnnotationResult<()> {
    if let ValueSpec::Discrete { values, .. } = spec {
        for (i, v) in values.iter().enumerate() {
            if let Scalar::String(s) = v
                && !allowed.contains(&s.as_str())
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

fn scalar_display(v: &Scalar) -> String {
    match v {
        Scalar::Int(i) => i.to_string(),
        Scalar::Double(d) => d.to_string(),
        Scalar::String(s) => format!("\"{s}\""),
        Scalar::Bool(b) => b.to_string(),
    }
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
