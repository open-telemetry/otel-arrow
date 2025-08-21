// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{AsStaticValue, StaticScalarExpression, StaticValue};

#[derive(Debug)]
pub enum ResolvedStaticScalarExpression<'a> {
    Reference(&'a StaticScalarExpression),
    Value(StaticScalarExpression),
}

impl AsStaticValue for ResolvedStaticScalarExpression<'_> {
    fn to_static_value(&self) -> StaticValue<'_> {
        match self {
            ResolvedStaticScalarExpression::Reference(s) => s.to_static_value(),
            ResolvedStaticScalarExpression::Value(s) => s.to_static_value(),
        }
    }
}

#[cfg(test)]
impl AsRef<StaticScalarExpression> for ResolvedStaticScalarExpression<'_> {
    fn as_ref(&self) -> &StaticScalarExpression {
        match self {
            ResolvedStaticScalarExpression::Reference(s) => s,
            ResolvedStaticScalarExpression::Value(s) => s,
        }
    }
}
