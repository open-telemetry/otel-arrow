// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

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

impl MutableValueExpression {
    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        match self {
            MutableValueExpression::Source(s) => s.try_fold(scope),
            MutableValueExpression::Variable(v) => v.try_fold(scope),
        }
    }
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
