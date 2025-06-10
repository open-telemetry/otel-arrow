use std::{cell::OnceCell, collections::LinkedList};

use crate::{
    data_expressions::data_expression::*, error::Error, execution_context::ExecutionContext,
    expression::*,
};

#[derive(Debug)]
pub struct PipelineExpression {
    id: usize,
    expressions: LinkedList<Box<dyn DataExpression>>,
    hash: OnceCell<ExpressionHash>,
}

impl Expression for PipelineExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            return ExpressionHash::new(|h| {
                h.add_bytes(b"pipeline");
                if self.expressions.len() > 0 {
                    h.add_bytes(b"expressions:");
                    for expression in self.expressions.iter() {
                        h.add_bytes(expression.get_hash().get_bytes());
                    }
                }
            });
        })
    }

    fn write_debug(
        &self,
        execution_context: &dyn ExecutionContext,
        heading: &'static str,
        level: i32,
        output: &mut String,
    ) {
        let padding = "\t".repeat(level as usize);

        output.push_str(&padding);
        output.push_str(heading);
        output.push_str("[\n");

        let mut is_first = true;
        for expression in self.expressions.iter() {
            if !is_first {
                output.push_str(&padding);
                output.push_str(" ,\n");
            } else {
                is_first = false;
            }
            expression.write_debug(execution_context, "", level + 1, output);
        }

        output.push_str(&padding);
        output.push_str("]\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl PipelineExpression {
    pub fn new() -> PipelineExpression {
        Self {
            id: get_next_id(),
            expressions: LinkedList::new(),
            hash: OnceCell::new(),
        }
    }

    pub fn add_data_expression(&mut self, expression: impl DataExpression + 'static) {
        self.expressions.push_back(Box::new(expression));
    }
}

impl PipelineExpression {
    pub(crate) fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<DataExpressionResult, Error>
    where
        'a: 'b,
    {
        for expression in self.expressions.iter() {
            let result = execution_context.evaluate(expression.as_ref())?;
            if let DataExpressionResult::None = result {
                continue;
            }

            return Ok(result);
        }

        return Ok(DataExpressionResult::Include(self.get_id()));
    }
}
