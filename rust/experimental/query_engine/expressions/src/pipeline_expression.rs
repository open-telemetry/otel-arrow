// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{DataExpression, Expression, ExpressionError, QueryLocation, StaticScalarExpression};

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExpression {
    query: Box<str>,
    query_location: QueryLocation,
    constants: Vec<StaticScalarExpression>,
    expressions: Vec<DataExpression>,
}

impl PipelineExpression {
    pub(crate) fn new(query: &str) -> PipelineExpression {
        Self {
            query: query.into(),
            query_location: QueryLocation::new(0, query.len(), 1, 1).unwrap(),
            constants: Vec::new(),
            expressions: Vec::new(),
        }
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }

    pub fn get_query_slice(&self, query_location: &QueryLocation) -> &str {
        let (start, end) = query_location.get_start_and_end_positions();

        &self.query[start..end]
    }

    pub fn push_constant(&mut self, value: StaticScalarExpression) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn get_constants(&self) -> &Vec<StaticScalarExpression> {
        &self.constants
    }

    pub fn get_constant(&self, constant_id: usize) -> Option<&StaticScalarExpression> {
        self.constants.get(constant_id)
    }

    pub fn get_expressions(&self) -> &[DataExpression] {
        &self.expressions
    }

    pub(crate) fn push_expression(&mut self, expression: DataExpression) {
        self.expressions.push(expression);
    }

    pub(crate) fn optimize(&mut self) -> Result<(), Vec<ExpressionError>> {
        // todo: Implement constant folding and other optimizations
        Ok(())
    }
}

impl Default for PipelineExpression {
    fn default() -> Self {
        PipelineExpression::new("")
    }
}

impl Expression for PipelineExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "PipelineExpression"
    }
}

pub struct PipelineExpressionBuilder {
    pipeline: PipelineExpression,
}

impl PipelineExpressionBuilder {
    pub fn new(query: &str) -> PipelineExpressionBuilder {
        Self {
            pipeline: PipelineExpression::new(query),
        }
    }

    pub fn with_constants(
        mut self,
        constants: Vec<StaticScalarExpression>,
    ) -> PipelineExpressionBuilder {
        for c in constants {
            self.push_constant(c);
        }

        self
    }

    pub fn with_expressions(
        mut self,
        expressions: Vec<DataExpression>,
    ) -> PipelineExpressionBuilder {
        for expression in expressions {
            self.push_expression(expression);
        }

        self
    }

    pub fn push_expression(&mut self, expression: DataExpression) {
        self.pipeline.push_expression(expression);
    }

    pub fn push_constant(&mut self, value: StaticScalarExpression) -> usize {
        self.pipeline.push_constant(value)
    }

    pub fn build(self) -> Result<PipelineExpression, Vec<ExpressionError>> {
        let mut p = self.pipeline;
        p.optimize()?;
        Ok(p)
    }
}

impl AsRef<PipelineExpression> for PipelineExpressionBuilder {
    fn as_ref(&self) -> &PipelineExpression {
        &self.pipeline
    }
}
