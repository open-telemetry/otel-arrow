// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Handles mapping attributes defined in a resolved registry to a KeyValue pair for a otlp signal
//!
//! Attribute values are chosen in this order:
//!
//! 1. The `traffic-generator` annotation on the resolved attribute (which, for
//!    Weaver v1 references, is either the reference's or the definition's
//!    annotation).
//! 2. The first enum member.
//! 3. `examples`, sampled uniformly.
//! 4. A type default.
//!
//! Each attribute is compiled once into an [`AttributeGen`] when the registry
//! is loaded: the annotation is parsed, its samplers are built, and the
//! fallback candidates (enum members, examples, or the type default) are
//! converted to `AnyValue`s. Generating a value only draws from the compiled
//! form.

use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use rand::Rng;
use rand::seq::IndexedRandom;
use weaver_resolved_schema::v1::attribute::Attribute;
use weaver_semconv::v1::attribute::ValueSpec as SemconvValueSpec;
use weaver_semconv::v1::attribute::{
    AttributeType, Examples, PrimitiveOrArrayTypeSpec, TemplateTypeSpec,
};

use super::annotation::{
    self, AnnotationResult, AttributeSpec, Scalar, ValueSpec, scalar_to_string,
};
use super::sampler::{MappedSampler, NumberSampler, ValueSampler};

fn scalar_to_any(value: Scalar) -> AnyValue {
    match value {
        Scalar::Int(i) => AnyValue::new_int(i),
        Scalar::Double(d) => AnyValue::new_double(d),
        Scalar::String(s) => AnyValue::new_string(s),
        Scalar::Bool(b) => AnyValue::new_bool(b),
    }
}

fn semconv_to_any(value: &SemconvValueSpec) -> AnyValue {
    match value {
        SemconvValueSpec::Int(i) => AnyValue::new_int(*i),
        SemconvValueSpec::Double(d) => AnyValue::new_double(f64::from(*d)),
        SemconvValueSpec::String(s) => AnyValue::new_string(s.clone()),
        SemconvValueSpec::Bool(b) => AnyValue::new_bool(*b),
    }
}

/// Whether continuous samples for this attribute type should be integers.
const fn is_int_type(ty: &AttributeType) -> bool {
    match ty {
        AttributeType::PrimitiveOrArray(spec) => {
            matches!(
                spec,
                PrimitiveOrArrayTypeSpec::Int | PrimitiveOrArrayTypeSpec::Ints
            )
        }
        AttributeType::Template(spec) => {
            matches!(spec, TemplateTypeSpec::Int | TemplateTypeSpec::Ints)
        }
        AttributeType::Enum { .. } => false,
    }
}

// ---------------------------------------------------------------------------
// Compiled attribute generators
// ---------------------------------------------------------------------------

/// Compiled generator for a single scalar value.
#[derive(Debug, Clone)]
enum ScalarGen {
    /// Discrete values, already converted to `AnyValue`.
    Choice(MappedSampler<AnyValue>),
    /// Continuous range.
    Number(NumberSampler),
}

impl ScalarGen {
    fn new(spec: &ValueSpec, integer: bool) -> Self {
        match spec {
            ValueSpec::Continuous {
                range,
                distribution,
            } => ScalarGen::Number(NumberSampler::new(*range, *distribution, integer)),
            // Every scalar maps to an `AnyValue`, so the infallible
            // constructor applies.
            ValueSpec::Discrete { values, weights } => ScalarGen::Choice(
                MappedSampler::from_discrete_parts(values, weights.as_deref(), |v| {
                    scalar_to_any(v.clone())
                }),
            ),
        }
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AnyValue {
        match self {
            ScalarGen::Choice(choice) => choice.sample(rng).clone(),
            ScalarGen::Number(sampler) => {
                let x = sampler.sample_f64(rng);
                if sampler.is_integer() {
                    AnyValue::new_int(x as i64)
                } else {
                    AnyValue::new_double(x)
                }
            }
        }
    }
}

/// Compiled generator for an attribute value.
#[derive(Debug, Clone)]
enum ValueGen {
    /// Annotated scalar.
    Scalar(ScalarGen),
    /// Annotated array: a length and an item generator.
    Array {
        length: ValueSampler,
        items: ScalarGen,
    },
    /// Non-annotated attribute: a non-empty list of candidate values
    /// (enum members, examples, or a type default) picked uniformly.
    Fallback(Vec<AnyValue>),
}

impl ValueGen {
    fn from_spec(spec: &AttributeSpec, integer: bool) -> Self {
        match spec {
            AttributeSpec::Scalar(v) => ValueGen::Scalar(ScalarGen::new(v, integer)),
            AttributeSpec::Array { length, items } => ValueGen::Array {
                length: ValueSampler::new(length, true),
                items: ScalarGen::new(items, integer),
            },
            AttributeSpec::Template { value, .. } => ValueGen::from_spec(value, integer),
        }
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AnyValue {
        match self {
            ValueGen::Scalar(s) => s.sample(rng),
            ValueGen::Array { length, items } => {
                let n = length.sample_count(rng);
                AnyValue::new_array((0..n).map(|_| items.sample(rng)).collect::<Vec<_>>())
            }
            ValueGen::Fallback(candidates) => match candidates.as_slice() {
                [only] => only.clone(),
                many => many
                    .choose(rng)
                    .cloned()
                    .unwrap_or_else(|| AnyValue::new_string("value")),
            },
        }
    }
}

/// Compiled generator for an attribute key.
#[derive(Debug, Clone)]
enum NameGen {
    /// The attribute name.
    Fixed(String),
    /// Template attribute: precomputed `<prefix>.<key>` names.
    Keys(MappedSampler<String>),
}

/// A registry attribute compiled into a reusable name/value generator.
#[derive(Debug, Clone)]
pub struct AttributeGen {
    name: NameGen,
    value: ValueGen,
}

impl AttributeGen {
    /// Compile an attribute.
    ///
    /// Uses the `traffic-generator` annotation when present; otherwise the
    /// enum members, examples, or type default. Returns an error when the
    /// annotation is invalid.
    pub fn try_new(attribute: &Attribute) -> AnnotationResult<Self> {
        match annotation::lookup(attribute.annotations.as_ref())? {
            Some(map) => {
                let spec = annotation::parse_attribute_spec(map, &attribute.r#type)?;
                Ok(Self::from_spec(attribute, &spec))
            }
            None => Ok(Self::fallback(attribute)),
        }
    }

    /// Compile an attribute from an already parsed annotation spec.
    #[must_use]
    pub fn from_spec(attribute: &Attribute, spec: &AttributeSpec) -> Self {
        let integer = is_int_type(&attribute.r#type);
        let name = match spec {
            AttributeSpec::Template { keys, .. } => MappedSampler::new(keys, |k| {
                Some(format!("{}.{}", attribute.name, scalar_to_string(k)))
            })
            .map_or_else(|| NameGen::Fixed(attribute.name.clone()), NameGen::Keys),
            _ => NameGen::Fixed(attribute.name.clone()),
        };
        Self {
            name,
            value: ValueGen::from_spec(spec, integer),
        }
    }

    /// Compile the non-annotated behavior (enum members, examples, type default).
    #[must_use]
    pub fn fallback(attribute: &Attribute) -> Self {
        Self {
            name: NameGen::Fixed(attribute.name.clone()),
            value: ValueGen::Fallback(fallback_candidates(attribute)),
        }
    }

    /// Generate one name/value pair.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> KeyValue {
        // The value is drawn before the template key; seeded tests depend on
        // this draw order.
        let value = self.value.sample(rng);
        let name = match &self.name {
            NameGen::Fixed(n) => n.clone(),
            NameGen::Keys(keys) => keys.sample(rng).clone(),
        };
        KeyValue::new(name, value)
    }
}

// ---------------------------------------------------------------------------
// Fallback candidates
// ---------------------------------------------------------------------------

/// `items` converted with `f`, or `[default]` when `items` is empty.
fn or_default<T>(items: &[T], f: impl Fn(&T) -> AnyValue, default: AnyValue) -> Vec<AnyValue> {
    if items.is_empty() {
        vec![default]
    } else {
        items.iter().map(f).collect()
    }
}

fn int_array(v: &[i64]) -> AnyValue {
    AnyValue::new_array(v.iter().map(|i| AnyValue::new_int(*i)).collect::<Vec<_>>())
}

fn double_array<T: Copy + Into<f64>>(v: &[T]) -> AnyValue {
    AnyValue::new_array(
        v.iter()
            .map(|d| AnyValue::new_double((*d).into()))
            .collect::<Vec<_>>(),
    )
}

fn bool_array(v: &[bool]) -> AnyValue {
    AnyValue::new_array(v.iter().map(|b| AnyValue::new_bool(*b)).collect::<Vec<_>>())
}

fn string_array(v: &[String]) -> AnyValue {
    AnyValue::new_array(
        v.iter()
            .map(|s| AnyValue::new_string(s.clone()))
            .collect::<Vec<_>>(),
    )
}

/// Candidate values used when an attribute has no annotation.
///
/// One candidate is picked uniformly per generated value. The list is never
/// empty.
fn fallback_candidates(attribute: &Attribute) -> Vec<AnyValue> {
    let ex = attribute.examples.as_ref();
    match &attribute.r#type {
        AttributeType::PrimitiveOrArray(primitive_or_array) => match primitive_or_array {
            PrimitiveOrArrayTypeSpec::Boolean => vec![AnyValue::new_bool(true)],
            PrimitiveOrArrayTypeSpec::Int => match ex {
                Some(Examples::Int(i)) => vec![AnyValue::new_int(*i)],
                Some(Examples::Ints(ints)) => {
                    or_default(ints, |i| AnyValue::new_int(*i), AnyValue::new_int(42))
                }
                _ => vec![AnyValue::new_int(42)],
            },
            PrimitiveOrArrayTypeSpec::Double => match ex {
                Some(Examples::Double(d)) => vec![AnyValue::new_double(f64::from(*d))],
                Some(Examples::Doubles(doubles)) => or_default(
                    doubles,
                    |d| AnyValue::new_double(f64::from(*d)),
                    AnyValue::new_double(3.13),
                ),
                _ => vec![AnyValue::new_double(3.13)],
            },
            PrimitiveOrArrayTypeSpec::String => match ex {
                Some(Examples::String(s)) => vec![AnyValue::new_string(s.clone())],
                Some(Examples::Strings(strings)) => or_default(
                    strings,
                    |s| AnyValue::new_string(s.clone()),
                    AnyValue::new_string("value"),
                ),
                _ => vec![AnyValue::new_string("value")],
            },
            PrimitiveOrArrayTypeSpec::Any => match ex {
                // Boolean-based examples
                Some(Examples::Bool(b)) => vec![AnyValue::new_bool(*b)],
                Some(Examples::Bools(booleans)) => or_default(
                    booleans,
                    |b| AnyValue::new_bool(*b),
                    AnyValue::new_bool(true),
                ),
                Some(Examples::ListOfBools(lists)) => {
                    or_default(lists, |l| bool_array(l), bool_array(&[true, false]))
                }
                // Integer-based examples
                Some(Examples::Int(i)) => vec![AnyValue::new_int(*i)],
                Some(Examples::Ints(ints)) => {
                    or_default(ints, |i| AnyValue::new_int(*i), AnyValue::new_int(42))
                }
                Some(Examples::ListOfInts(lists)) => {
                    or_default(lists, |l| int_array(l), int_array(&[42, 43]))
                }
                // Double-based examples
                Some(Examples::Double(d)) => vec![AnyValue::new_double(f64::from(*d))],
                Some(Examples::Doubles(doubles)) => or_default(
                    doubles,
                    |d| AnyValue::new_double(f64::from(*d)),
                    AnyValue::new_double(3.13),
                ),
                Some(Examples::ListOfDoubles(lists)) => {
                    or_default(lists, |l| double_array(l), double_array(&[3.13, 3.15]))
                }
                // String-based examples
                Some(Examples::String(s)) => vec![AnyValue::new_string(s.clone())],
                Some(Examples::Strings(strings)) => or_default(
                    strings,
                    |s| AnyValue::new_string(s.clone()),
                    AnyValue::new_string("value"),
                ),
                Some(Examples::ListOfStrings(lists)) => or_default(
                    lists,
                    |l| string_array(l),
                    string_array(&["value1".to_owned(), "value2".to_owned()]),
                ),
                Some(Examples::Any(any)) => vec![semconv_to_any(any)],
                Some(Examples::Anys(anys)) => {
                    or_default(anys, semconv_to_any, AnyValue::new_string("value"))
                }
                // Fallback to a default value
                None => vec![AnyValue::new_string("value")],
            },
            PrimitiveOrArrayTypeSpec::Booleans => vec![bool_array(&[true, false])],
            PrimitiveOrArrayTypeSpec::Ints => match ex {
                Some(Examples::Ints(ints)) => vec![int_array(ints)],
                Some(Examples::ListOfInts(lists)) => {
                    or_default(lists, |l| int_array(l), int_array(&[42, 43]))
                }
                _ => vec![int_array(&[42, 43])],
            },
            PrimitiveOrArrayTypeSpec::Doubles => match ex {
                Some(Examples::Doubles(doubles)) => vec![double_array(doubles)],
                Some(Examples::ListOfDoubles(lists)) => {
                    or_default(lists, |l| double_array(l), double_array(&[3.13, 3.15]))
                }
                _ => vec![double_array(&[3.13, 3.15])],
            },
            PrimitiveOrArrayTypeSpec::Strings => match ex {
                Some(Examples::Strings(strings)) => vec![string_array(strings)],
                Some(Examples::ListOfStrings(lists)) => or_default(
                    lists,
                    |l| string_array(l),
                    string_array(&["value1".to_owned(), "value2".to_owned()]),
                ),
                _ => vec![string_array(&["value1".to_owned(), "value2".to_owned()])],
            },
        },
        // The first member only; annotate the attribute to vary members.
        AttributeType::Enum { members, .. } => vec![members.first().map_or_else(
            || AnyValue::new_string("value"),
            |m| semconv_to_any(&m.value),
        )],
        AttributeType::Template(template_type_spec) => {
            // TODO Support examples when https://github.com/open-telemetry/semantic-conventions/issues/1740 is complete
            vec![match template_type_spec {
                TemplateTypeSpec::String => AnyValue::new_string("template_value"),
                TemplateTypeSpec::Int => AnyValue::new_int(42),
                TemplateTypeSpec::Double => AnyValue::new_double(3.13),
                TemplateTypeSpec::Boolean => AnyValue::new_bool(true),
                TemplateTypeSpec::Any => AnyValue::new_string("template_any_value"),
                TemplateTypeSpec::Strings => {
                    string_array(&["template_value1".to_owned(), "template_value2".to_owned()])
                }
                TemplateTypeSpec::Ints => int_array(&[42, 43]),
                TemplateTypeSpec::Doubles => double_array(&[3.13, 3.15]),
                TemplateTypeSpec::Booleans => bool_array(&[true, false]),
            }]
        }
    }
}
