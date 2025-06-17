use std::cell::OnceCell;

use crate::{
    error::Error, execution_context::ExecutionContext, expression::*,
    logical_expressions::logical_expression::LogicalExpression,
    transform_expressions::transformation_expression::TransformationExpression,
};

use super::data_expression::{DataExpressionInternal, DataExpressionResult};

#[derive(Debug)]
pub struct TransformDataExpression {
    id: usize,
    transformations: Vec<Box<dyn TransformationExpression>>,
    predicate: Option<Box<dyn LogicalExpression>>,
    hash: OnceCell<ExpressionHash>,
}

impl Expression for TransformDataExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"transform");
                if self.predicate.is_some() {
                    h.add_bytes(b"predicate:");
                    h.add_bytes(self.predicate.as_ref().unwrap().get_hash().get_bytes());
                }
                h.add_bytes(b"transformations:");
                for transformation_expr in self.transformations.iter() {
                    h.add_bytes(transformation_expr.get_hash().get_bytes());
                }
            })
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
        output.push_str("transform (\n");

        if self.predicate.is_some() {
            self.predicate.as_ref().unwrap().write_debug(
                execution_context,
                "predicate: ",
                level + 1,
                output,
            );

            output.push_str(&padding);
            output.push_str(" ,\n");
        }

        output.push_str(&padding);
        output.push_str("\ttransformations: [\n");

        let mut is_first = true;
        for transformation_expr in self.transformations.iter() {
            if !is_first {
                output.push_str(&padding);
                output.push_str("\t ,\n");
            } else {
                is_first = false;
            }
            transformation_expr.write_debug(execution_context, "", level + 2, output);
        }

        output.push_str(&padding);
        output.push_str("\t]\n");

        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl DataExpressionInternal for TransformDataExpression {
    fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<DataExpressionResult, Error>
    where
        'a: 'b,
    {
        if self.predicate.is_some()
            && !self
                .predicate
                .as_ref()
                .unwrap()
                .evaluate(execution_context)?
        {
            execution_context.add_message_for_expression(
                self,
                ExpressionMessage::info("TransformDataExpression evaluation skipped".to_string()),
            );

            return Ok(DataExpressionResult::None);
        }

        for transformation_expr in self.transformations.iter() {
            transformation_expr.evaluate(execution_context)?;
        }

        execution_context.add_message_for_expression(
            self,
            ExpressionMessage::info("TransformDataExpression evaluated".to_string()),
        );

        Ok(DataExpressionResult::None)
    }
}

impl Default for TransformDataExpression {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformDataExpression {
    pub fn new() -> TransformDataExpression {
        Self {
            id: get_next_id(),
            transformations: Vec::new(),
            predicate: None,
            hash: OnceCell::new(),
        }
    }

    pub fn new_with_predicate(
        predicate: impl LogicalExpression + 'static,
    ) -> TransformDataExpression {
        Self {
            id: get_next_id(),
            transformations: Vec::new(),
            predicate: Some(Box::new(predicate)),
            hash: OnceCell::new(),
        }
    }

    pub fn add_transformation_expression(
        &mut self,
        transformation: impl TransformationExpression + 'static,
    ) {
        self.transformations.push(Box::new(transformation));
    }
}
