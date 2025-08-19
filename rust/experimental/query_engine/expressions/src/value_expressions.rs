// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Expression, QueryLocation, ScalarExpression, SourceScalarExpression, VariableScalarExpression,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MutableValueExpression {
    /// Source value.
    ///
    /// Note: Source may refer to the source itself (root) or data on the source
    /// accessed via [`crate::ValueAccessor`] selectors.
    Source(SourceScalarExpression),

    /// Variable value.
    ///
    /// Note: Variable may refer to the variable itself (root) or data on the
    /// variable accessed via [`crate::ValueAccessor`] selectors.
    Variable(VariableScalarExpression),
}

impl Expression for MutableValueExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            MutableValueExpression::Source(s) => s.get_query_location(),
            MutableValueExpression::Variable(v) => v.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            MutableValueExpression::Source(_) => "MutableValueExpression(Source)",
            MutableValueExpression::Variable(_) => "MutableValueExpression(Variable)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImmutableValueExpression {
    /// Scalar value.
    Scalar(ScalarExpression),
}

impl Expression for ImmutableValueExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ImmutableValueExpression::Scalar(s) => s.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ImmutableValueExpression::Scalar(_) => "ImmutableValueExpression(Scalar)",
        }
    }
}
