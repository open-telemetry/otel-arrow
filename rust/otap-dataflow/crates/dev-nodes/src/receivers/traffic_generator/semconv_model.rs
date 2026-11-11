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
//! [`SemconvModel::compile_without_annotations`] skips annotations entirely
//! (the receiver's `annotations: false` mode): values come from enum members,
//! examples, and type defaults only.
//!
//! [`SemconvModel::compile`]: crate::receivers::traffic_generator::semconv_model::SemconvModel::compile
//! [`SemconvModel::compile_without_annotations`]: crate::receivers::traffic_generator::semconv_model::SemconvModel::compile_without_annotations

use weaver_forge::v1::registry::{ResolvedGroup, ResolvedRegistry};
use weaver_semconv::v1::group::{GroupType, InstrumentSpec, SpanKindSpec};

use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::AggregationTemporality;

use super::annotation::{
    self, AnnotationError, AnnotationResult, EventSpec, MetricSpec, NAMESPACE,
    RegistryAnnotationError, SpanSpec, scalar_to_string,
};
use super::attributes::AttributeGen;
use super::fake_data::HISTOGRAM_BOUNDS;
use super::sampler::{MappedSampler, ValueSampler};

/// A span group compiled for generation.
#[derive(Debug, Clone)]
pub struct SpanTemplate {
    /// Span name (the group id).
    pub(crate) name: String,
    /// Span kind from the registry.
    pub(crate) kind: Option<SpanKindSpec>,
    /// Names of the span events.
    pub(crate) events: Vec<String>,
    /// Attribute generators.
    pub(crate) attributes: Vec<AttributeGen>,
    /// Duration sampler in milliseconds (`duration` annotation).
    pub(crate) duration_ms: Option<ValueSampler>,
    /// Status code sampler (`status` annotation).
    pub(crate) status: Option<MappedSampler<i32>>,
}

/// A metric group compiled for generation.
#[derive(Debug, Clone)]
pub struct MetricTemplate {
    /// Metric name.
    pub(crate) name: String,
    /// Metric description (the group brief).
    pub(crate) description: String,
    /// Metric unit.
    pub(crate) unit: String,
    /// Instrument kind.
    pub(crate) instrument: InstrumentSpec,
    /// Attribute generators.
    pub(crate) attributes: Vec<AttributeGen>,
    /// Value sampler (`value`) for number points and histogram observations.
    pub(crate) value: Option<ValueSampler>,
    /// Observations per histogram data point (`observations`).
    pub(crate) observations: Option<ValueSampler>,
    /// Histogram bucket bounds (`bounds`, or the SDK defaults).
    pub(crate) bounds: Vec<f64>,
    /// How histogram data points are populated.
    pub(crate) histogram_mode: HistogramMode,
    /// Sum and histogram temporality (`temporality`, or `Unspecified`).
    pub(crate) temporality: AggregationTemporality,
}

/// How histogram data points are populated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistogramMode {
    /// Sample observations into `bounds` and emit bucket counts, count, sum,
    /// min, and max (annotation-driven generation).
    #[default]
    Aggregated,
    /// Emit points with empty bucket counts and bounds and no count, sum,
    /// min, or max (generation with annotations disabled).
    Empty,
}

/// An event group compiled for generation.
#[derive(Debug, Clone)]
pub struct EventTemplate {
    /// Event name.
    pub(crate) name: String,
    /// Log body text.
    pub(crate) body: String,
    /// Attribute generators.
    pub(crate) attributes: Vec<AttributeGen>,
    /// Severity number and text sampler (`severity` annotation).
    pub(crate) severity: Option<MappedSampler<(i32, &'static str)>>,
}

/// A resolved registry compiled into signal templates.
#[derive(Debug, Clone, Default)]
pub struct SemconvModel {
    /// Span groups, in registry order.
    pub(crate) spans: Vec<SpanTemplate>,
    /// Metric groups with an instrument, in registry order.
    pub(crate) metrics: Vec<MetricTemplate>,
    /// Event groups, in registry order.
    pub(crate) events: Vec<EventTemplate>,
    /// Number of `traffic-generator` annotations found while compiling (0 when
    /// compiled without annotations). Used to warn when `annotations: true` but
    /// the registry carries none.
    annotation_count: usize,
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
            model.annotation_count += count_group_annotations(group);
            match compile_group(group, attributes, true) {
                Ok(Some(template)) => model.push(template),
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

    /// Compile `registry` ignoring every `traffic-generator` annotation.
    ///
    /// Annotations are never read, so invalid annotations cannot fail.
    /// Attribute values come from enum members, examples, and type defaults;
    /// signal-level settings use their defaults, and histogram points are
    /// emitted without buckets ([`HistogramMode::Empty`]).
    #[must_use]
    pub fn compile_without_annotations(registry: &ResolvedRegistry) -> Self {
        let mut model = SemconvModel::default();
        for group in &registry.groups {
            let attributes = group
                .attributes
                .iter()
                .map(AttributeGen::fallback)
                .collect();
            // With annotations disabled no annotation is read, so this never
            // returns an error.
            if let Ok(Some(template)) = compile_group(group, attributes, false) {
                model.push(template);
            }
        }
        model
    }

    /// Number of `traffic-generator` annotations found while compiling (0 when
    /// compiled without annotations).
    #[must_use]
    pub fn annotation_count(&self) -> usize {
        self.annotation_count
    }

    fn push(&mut self, template: Template) {
        match template {
            Template::Span(t) => self.spans.push(t),
            Template::Metric(t) => self.metrics.push(t),
            Template::Event(t) => self.events.push(t),
        }
    }
}

/// Count the `traffic-generator` annotations on a group and its attributes.
fn count_group_annotations(group: &ResolvedGroup) -> usize {
    let has = |a| matches!(annotation::lookup(a), Ok(Some(_)));
    usize::from(has(group.annotations.as_ref()))
        + group
            .attributes
            .iter()
            .filter(|attr| has(attr.annotations.as_ref()))
            .count()
}

enum Template {
    Span(SpanTemplate),
    Metric(MetricTemplate),
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
    use_annotations: bool,
) -> AnnotationResult<Option<Template>> {
    // Both the group's annotation map and the histogram mode are decided by
    // `use_annotations`: with annotations off, no map is read and histograms
    // are emitted empty.
    let (map, histogram_mode) = if use_annotations {
        (
            annotation::lookup(group.annotations.as_ref())?,
            HistogramMode::Aggregated,
        )
    } else {
        (None, HistogramMode::Empty)
    };
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
                    .map(|s| named(s, "status", annotation::status_code))
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
                    .map(|s| named(s, "severity", annotation::severity))
                    .transpose()?,
            })))
        }
        (GroupType::Metric, Some(instrument)) => {
            let spec = match map {
                Some(m) => annotation::parse_metric_spec(m, instrument)?,
                None => MetricSpec::default(),
            };
            Ok(Some(Template::Metric(MetricTemplate {
                name: group.metric_name.clone().unwrap_or_default(),
                description: group.brief.clone(),
                unit: group.unit.clone().unwrap_or_default(),
                instrument: instrument.clone(),
                attributes,
                value: spec.value.as_ref().map(|v| ValueSampler::new(v, false)),
                observations: spec
                    .observations
                    .as_ref()
                    .map(|v| ValueSampler::new(v, true)),
                bounds: match histogram_mode {
                    HistogramMode::Aggregated => {
                        spec.bounds.unwrap_or_else(|| HISTOGRAM_BOUNDS.to_vec())
                    }
                    HistogramMode::Empty => Vec::new(),
                },
                histogram_mode,
                temporality: spec
                    .temporality
                    .unwrap_or(AggregationTemporality::Unspecified),
            })))
        }
        // Groups that produce no signal accept no annotation.
        _ if map.is_none() => Ok(None),
        (GroupType::Metric, None) => Err(AnnotationError::new(
            NAMESPACE,
            "annotations are not supported on metric groups without an instrument",
        )),
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
