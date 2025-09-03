// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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
    pub fn try_fold(self) -> Option<StaticScalarExpression> {
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
