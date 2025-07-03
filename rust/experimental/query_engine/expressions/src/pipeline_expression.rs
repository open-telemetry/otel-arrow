use crate::{DataExpression, Expression, ExpressionError, QueryLocation};

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExpression {
    query: Box<str>,
    query_location: QueryLocation,
    expressions: Vec<DataExpression>,
}

impl PipelineExpression {
    pub(crate) fn new(query: &str) -> PipelineExpression {
        Self {
            query: query.into(),
            query_location: QueryLocation::new(0, query.len(), 1, 1).unwrap(),
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

impl Expression for PipelineExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

pub struct PipelineExpressionBuilder {
    pipeline: PipelineExpression,
}

impl PipelineExpressionBuilder {
    pub fn new(query: &str) -> PipelineExpressionBuilder {
        PipelineExpressionBuilder::new_with_expressions(query, Vec::new())
    }

    pub fn new_with_expressions(
        query: &str,
        expressions: Vec<DataExpression>,
    ) -> PipelineExpressionBuilder {
        let mut s = Self {
            pipeline: PipelineExpression::new(query),
        };

        for expression in expressions {
            s.push_expression(expression);
        }

        s
    }

    pub fn push_expression(&mut self, expression: DataExpression) {
        self.pipeline.push_expression(expression);
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
