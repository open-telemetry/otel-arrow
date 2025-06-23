use crate::{DataExpression, Expression, QueryLocation};

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExpression {
    query_location: QueryLocation,
    expressions: Vec<DataExpression>,
}

impl PipelineExpression {
    pub fn new(query_location: QueryLocation) -> PipelineExpression {
        Self {
            query_location,
            expressions: Vec::new(),
        }
    }

    pub fn new_with_expressions(
        query_location: QueryLocation,
        expressions: Vec<DataExpression>,
    ) -> PipelineExpression {
        Self {
            query_location,
            expressions,
        }
    }

    pub fn push_expression(&mut self, expression: DataExpression) {
        self.expressions.push(expression);
    }
}

impl Expression for PipelineExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}
