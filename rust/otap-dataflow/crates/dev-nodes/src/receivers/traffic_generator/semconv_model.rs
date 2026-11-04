// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Semantic conventions registry compiled for signal generation.
//!
//! [`SemconvModel::compile`] walks a resolved registry once: it parses and
//! validates every `traffic-generator` annotation, builds every sampler
//! (value, count, duration, status, severity, attribute), and copies the
//! group fields the generator emits (names, units, span kinds, ...). Signal
//! generation then only draws from the compiled templates; no YAML is read
//! and no sampler is rebuilt per signal.
//!
//! Compilation reports every invalid annotation (group, attribute, key path),
//! so it is also the registry validation pass: an invalid annotation fails
//! receiver startup.
//!
//! [`SemconvModel::compile`]: crate::receivers::traffic_generator::semconv_model::SemconvModel::compile

use weaver_forge::v1::registry::{ResolvedGroup, ResolvedRegistry};
use weaver_semconv::v1::group::{GroupType, InstrumentSpec, SpanKindSpec};

use super::annotation::{
    self, AnnotationError, AnnotationResult, DEFAULT_HISTOGRAM_BOUNDS, EventSpec, MetricSpec,
    NAMESPACE, RegistryAnnotationError, SEVERITY_NAMES, STATUS_NAMES, SpanSpec,
};
use super::attributes::AttributeGen;
use super::sampler::{MappedSampler, ValueSampler, scalar_to_string};

/// A span group compiled for generation.
#[derive(Debug, Clone)]
pub struct SpanTemplate {
    /// Span name (the group id).
    pub name: String,
    /// Span kind from the registry.
    pub kind: Option<SpanKindSpec>,
    /// Names of the span events.
    pub events: Vec<String>,
    /// Attribute generators.
    pub attributes: Vec<AttributeGen>,
    /// Duration sampler in milliseconds (`duration` annotation).
    pub duration_ms: Option<ValueSampler>,
    /// Status code sampler (`status` annotation).
    pub status: Option<MappedSampler<i32>>,
}

/// A metric group compiled for generation.
#[derive(Debug, Clone)]
pub struct MetricTemplate {
    /// Metric name.
    pub name: String,
    /// Metric description (the group brief).
    pub description: String,
    /// Metric unit.
    pub unit: String,
    /// Instrument kind.
    pub instrument: InstrumentSpec,
    /// Number points are emitted as `as_int` (`code_generation.metric_value_type: int`).
    pub int_value: bool,
    /// Attribute generators.
    pub attributes: Vec<AttributeGen>,
    /// Value sampler (`value`); integer output for int number metrics,
    /// double output for histogram observations.
    pub value: Option<ValueSampler>,
    /// Data points per metric (`data_points`).
    pub data_points: Option<ValueSampler>,
    /// Observations per histogram data point (`observations`).
    pub observations: Option<ValueSampler>,
    /// Histogram bucket bounds (`bounds`, or the SDK defaults).
    pub bounds: Vec<f64>,
}

/// An event group compiled for generation.
#[derive(Debug, Clone)]
pub struct EventTemplate {
    /// Event name.
    pub name: String,
    /// Log body text.
    pub body: String,
    /// Attribute generators.
    pub attributes: Vec<AttributeGen>,
    /// Severity number and text sampler (`severity` annotation).
    pub severity: Option<MappedSampler<(i32, &'static str)>>,
}

/// A resolved registry compiled into signal templates.
#[derive(Debug, Clone, Default)]
pub struct SemconvModel {
    /// Span groups, in registry order.
    pub spans: Vec<SpanTemplate>,
    /// Metric groups, in registry order. `None` for metric groups without an
    /// instrument: they still occupy a slot in the rotation (and are skipped
    /// when generating), so the metric rotation follows registry order.
    pub metrics: Vec<Option<MetricTemplate>>,
    /// Event groups, in registry order.
    pub events: Vec<EventTemplate>,
}

/// Map a severity short name to its SeverityNumber value (1..=24) and its
/// static name.
pub(crate) fn severity_entry(name: &str) -> Option<(i32, &'static str)> {
    SEVERITY_NAMES
        .iter()
        .position(|n| *n == name)
        .map(|i| (i as i32 + 1, SEVERITY_NAMES[i]))
}

/// Map a status name to its StatusCode value.
pub(crate) fn status_entry(name: &str) -> Option<i32> {
    STATUS_NAMES
        .iter()
        .position(|n| *n == name)
        .map(|i| i as i32)
}

impl SemconvModel {
    /// Compile `registry`, validating every `traffic-generator` annotation.
    ///
    /// Returns every invalid annotation (group, attribute, key path) so they
    /// can be reported together at startup.
    pub fn compile(registry: &ResolvedRegistry) -> Result<Self, Vec<RegistryAnnotationError>> {
        let mut model = SemconvModel::default();
        let mut errors = Vec::new();
        for group in &registry.groups {
            let mut attribute_errors = Vec::new();
            let attributes = compile_attributes(group, &mut attribute_errors);
            match compile_group(group, attributes) {
                Ok(Some(Template::Span(t))) => model.spans.push(t),
                Ok(Some(Template::Metric(t))) => model.metrics.push(t),
                Ok(Some(Template::Event(t))) => model.events.push(t),
                Ok(None) => {}
                Err(error) => errors.push(RegistryAnnotationError {
                    group: group.id.clone(),
                    attribute: None,
                    error,
                }),
            }
            // Group errors are listed before the group's attribute errors.
            errors.append(&mut attribute_errors);
        }
        if errors.is_empty() {
            Ok(model)
        } else {
            Err(errors)
        }
    }
}

enum Template {
    Span(SpanTemplate),
    Metric(Option<MetricTemplate>),
    Event(EventTemplate),
}

fn compile_attributes(
    group: &ResolvedGroup,
    errors: &mut Vec<RegistryAnnotationError>,
) -> Vec<AttributeGen> {
    group
        .attributes
        .iter()
        .filter_map(|attribute| match AttributeGen::try_new(attribute) {
            Ok(g) => Some(g),
            Err(error) => {
                errors.push(RegistryAnnotationError {
                    group: group.id.clone(),
                    attribute: Some(attribute.name.clone()),
                    error,
                });
                None
            }
        })
        .collect()
}

fn compile_group(
    group: &ResolvedGroup,
    attributes: Vec<AttributeGen>,
) -> AnnotationResult<Option<Template>> {
    let map = annotation::lookup(group.annotations.as_ref())?;
    match (&group.r#type, &group.instrument) {
        (GroupType::Span, _) => {
            let spec = match map {
                Some(m) => annotation::parse_span_spec(m)?,
                None => SpanSpec::default(),
            };
            Ok(Some(Template::Span(SpanTemplate {
                name: group.id.clone(),
                kind: group.span_kind.clone(),
                events: group.events.clone(),
                attributes,
                duration_ms: spec.duration.as_ref().map(|d| ValueSampler::new(d, false)),
                status: spec
                    .status
                    .as_ref()
                    .map(|s| named(s, "status", status_entry))
                    .transpose()?,
            })))
        }
        (GroupType::Event, _) => {
            let spec = match map {
                Some(m) => annotation::parse_event_spec(m)?,
                None => EventSpec::default(),
            };
            Ok(Some(Template::Event(EventTemplate {
                name: group.name.clone().unwrap_or_default(),
                body: group
                    .body
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                attributes,
                severity: spec
                    .severity
                    .as_ref()
                    .map(|s| named(s, "severity", severity_entry))
                    .transpose()?,
            })))
        }
        (GroupType::Metric, Some(instrument)) => {
            let spec = match map {
                Some(m) => annotation::parse_metric_spec(m, instrument)?,
                None => MetricSpec::default(),
            };
            let int_value = annotation::metric_value_is_int(group.annotations.as_ref());
            // Histogram observations are aggregated as doubles.
            let value_integer = int_value && !matches!(instrument, InstrumentSpec::Histogram);
            Ok(Some(Template::Metric(Some(MetricTemplate {
                name: group.metric_name.clone().unwrap_or_default(),
                description: group.brief.clone(),
                unit: group.unit.clone().unwrap_or_default(),
                instrument: instrument.clone(),
                int_value,
                attributes,
                value: spec
                    .value
                    .as_ref()
                    .map(|v| ValueSampler::new(v, value_integer)),
                data_points: spec
                    .data_points
                    .as_ref()
                    .map(|v| ValueSampler::new(v, true)),
                observations: spec
                    .observations
                    .as_ref()
                    .map(|v| ValueSampler::new(v, true)),
                bounds: spec
                    .bounds
                    .unwrap_or_else(|| DEFAULT_HISTOGRAM_BOUNDS.to_vec()),
            }))))
        }
        (GroupType::Metric, None) if map.is_none() => Ok(Some(Template::Metric(None))),
        _ if map.is_none() => Ok(None),
        _ => Err(AnnotationError::new(
            NAMESPACE,
            "annotations are only supported on metric, span, and event groups",
        )),
    }
}

/// Compile a discrete name spec (status, severity) into its mapped outputs.
fn named<T>(
    spec: &annotation::ValueSpec,
    what: &str,
    map: impl Fn(&str) -> Option<T>,
) -> AnnotationResult<MappedSampler<T>> {
    MappedSampler::new(spec, |v| map(&scalar_to_string(v))).ok_or_else(|| {
        // Unreachable after parsing (names are checked there); kept so a
        // parser gap surfaces as a startup error instead of a panic.
        AnnotationError::new(NAMESPACE, format!("invalid {what} values"))
    })
}
