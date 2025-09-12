// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use regex::Regex;

use crate::*;

#[derive(Debug)]
pub enum ResolvedStaticScalarExpression<'a> {
    /// A value computed by an expression.
    Computed(StaticScalarExpression),

    /// A reference to a static or a constant which cannot be folded.
    Reference(&'a StaticScalarExpression),

    /// A reference to a static which may be folded.
    FoldEligibleReference(&'a StaticScalarExpression),
}

impl ResolvedStaticScalarExpression<'_> {
    pub(crate) fn try_fold(self) -> Option<StaticScalarExpression> {
        match self {
            ResolvedStaticScalarExpression::Computed(c) => Some(c),
            ResolvedStaticScalarExpression::Reference(_) => {
                // Note: Don't copy referenced statics because if they were
                // foldable they would already have switched to values. For
                // example the reference could be to a large constant array.
                None
            }
            ResolvedStaticScalarExpression::FoldEligibleReference(r) => Some(r.clone()),
        }
    }

    pub(crate) fn try_resolve_static_regex<'a>(
        scalar: &'a mut ScalarExpression,
        scope: &PipelineResolutionScope<'a>,
    ) -> Result<Option<&'a RegexScalarExpression>, ExpressionError> {
        scalar.try_resolve_static(scope)?;

        match scalar {
            ScalarExpression::Static(StaticScalarExpression::Regex(r)) => Ok(Some(r)),
            ScalarExpression::Static(value) => {
                if !value.foldable() {
                    return Ok(None);
                }

                let mut result = None;

                value.to_value().convert_to_string(&mut |s| {
                    result = Some(Regex::new(s));
                });

                match result {
                    Some(Ok(r)) => {
                        let r = RegexScalarExpression::new(value.get_query_location().clone(), r);

                        *scalar = ScalarExpression::Static(StaticScalarExpression::Regex(r));

                        if let ScalarExpression::Static(StaticScalarExpression::Regex(r)) = scalar {
                            Ok(Some(r))
                        } else {
                            unreachable!()
                        }
                    }
                    Some(Err(e)) => Err(ExpressionError::ParseError(
                        value.get_query_location().clone(),
                        format!("Failed to parse Regex from pattern: {e}"),
                    )),
                    None => {
                        panic!(
                            "Encountered a Value which does not correctly implement convert_to_string"
                        )
                    }
                }
            }
            _ => Ok(None),
        }
    }
}

impl AsStaticValue for ResolvedStaticScalarExpression<'_> {
    fn to_static_value(&self) -> StaticValue<'_> {
        match self {
            ResolvedStaticScalarExpression::Computed(c) => c.to_static_value(),
            ResolvedStaticScalarExpression::Reference(r) => r.to_static_value(),
            ResolvedStaticScalarExpression::FoldEligibleReference(r) => r.to_static_value(),
        }
    }
}

impl AsRef<StaticScalarExpression> for ResolvedStaticScalarExpression<'_> {
    fn as_ref(&self) -> &StaticScalarExpression {
        match self {
            ResolvedStaticScalarExpression::Computed(c) => c,
            ResolvedStaticScalarExpression::Reference(r) => r,
            ResolvedStaticScalarExpression::FoldEligibleReference(r) => r,
        }
    }
}
