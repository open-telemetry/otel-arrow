// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug)]
pub enum ResolvedStaticScalarExpression<'a> {
    Reference(&'a StaticScalarExpression),
    Computed(StaticScalarExpression),
    FoldedConstant(&'a StaticScalarExpression),
}

impl ResolvedStaticScalarExpression<'_> {
    pub fn try_fold(self) -> Option<StaticScalarExpression> {
        match self {
            ResolvedStaticScalarExpression::Reference(_) => {
                // Note: Don't copy referenced statics because if they were
                // foldable they would already have switched to values. For
                // example the reference could be to a large constant array.
                None
            }
            ResolvedStaticScalarExpression::Computed(s) => Some(s),
            ResolvedStaticScalarExpression::FoldedConstant(s) => Some(s.clone()),
        }
    }
}

impl AsStaticValue for ResolvedStaticScalarExpression<'_> {
    fn to_static_value(&self) -> StaticValue<'_> {
        match self {
            ResolvedStaticScalarExpression::Reference(s) => s.to_static_value(),
            ResolvedStaticScalarExpression::Computed(s) => s.to_static_value(),
            ResolvedStaticScalarExpression::FoldedConstant(s) => s.to_static_value(),
        }
    }
}

#[cfg(test)]
impl AsRef<StaticScalarExpression> for ResolvedStaticScalarExpression<'_> {
    fn as_ref(&self) -> &StaticScalarExpression {
        match self {
            ResolvedStaticScalarExpression::Reference(s) => s,
            ResolvedStaticScalarExpression::Computed(s) => s,
            ResolvedStaticScalarExpression::FoldedConstant(s) => s,
        }
    }
}
