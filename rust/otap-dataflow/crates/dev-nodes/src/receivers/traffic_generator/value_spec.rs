// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared value-spec vocabulary for the traffic generator.
//!
//! These types describe a parsed `traffic-generator` annotation value. They are
//! the common vocabulary between the annotation parser ([`super::annotation`],
//! which produces them) and the samplers ([`super::sampler`], which consume
//! them). Keeping them in this leaf module -- which depends on neither -- lets
//! the parser call into the samplers for validation without a module cycle.

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
    pub(crate) const fn name(self) -> &'static str {
        match self {
            ScalarKind::Int => "int",
            ScalarKind::Double => "double",
            ScalarKind::String => "string",
            ScalarKind::Bool => "boolean",
            ScalarKind::Any => "any",
        }
    }

    pub(crate) const fn is_numeric(self) -> bool {
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

/// Format a scalar as a plain string (template keys, status and severity
/// names, error messages).
#[must_use]
pub fn scalar_to_string(value: &Scalar) -> String {
    match value {
        Scalar::String(s) => s.clone(),
        Scalar::Int(i) => i.to_string(),
        Scalar::Double(d) => d.to_string(),
        Scalar::Bool(b) => b.to_string(),
    }
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
