// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TemporalScalarExpression {
    /// Returns the current DateTime in the UTC time zone.
    Now(NowScalarExpression),
}

impl TemporalScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &mut self,
        _scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            TemporalScalarExpression::Now(_) => Ok(Some(ValueType::DateTime)),
        }
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        _scope: &PipelineResolutionScope,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        match self {
            TemporalScalarExpression::Now(_) => Ok(None),
        }
    }
}

impl Expression for TemporalScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            TemporalScalarExpression::Now(n) => n.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            TemporalScalarExpression::Now(_) => "TemporalScalar(Now)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NowScalarExpression {
    query_location: QueryLocation,
}

impl NowScalarExpression {
    pub fn new(query_location: QueryLocation) -> NowScalarExpression {
        Self { query_location }
    }
}

impl Expression for NowScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "NowScalarExpression"
    }
}
