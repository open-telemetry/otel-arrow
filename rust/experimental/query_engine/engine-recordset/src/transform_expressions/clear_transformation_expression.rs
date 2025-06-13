use std::cell::OnceCell;

use crate::{
    error::Error, execution_context::*, expression::*,
    logical_expressions::logical_expression::LogicalExpression,
};

use super::transformation_expression::TransformationExpressionInternal;

#[derive(Debug)]
pub struct ClearTransformationExpression {
    id: usize,
    predicate: Option<Box<dyn LogicalExpression>>,
    hash: OnceCell<ExpressionHash>,
}

impl Expression for ClearTransformationExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"clear");
                if self.predicate.is_some() {
                    h.add_bytes(b"predicate:");
                    h.add_bytes(self.predicate.as_ref().unwrap().get_hash().get_bytes());
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
        output.push_str("clear (\n");

        if self.predicate.is_some() {
            self.predicate.as_ref().unwrap().write_debug(
                execution_context,
                "predicate: ",
                level + 1,
                output,
            );
        }

        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl TransformationExpressionInternal for ClearTransformationExpression {
    fn evaluate<'a, 'b>(&'a self, execution_context: &dyn ExecutionContext<'b>) -> Result<(), Error>
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
                ExpressionMessage::info(
                    "ClearTransformationExpression evaluation skipped".to_string(),
                ),
            );

            return Ok(());
        }

        execution_context.clear();

        execution_context.add_message_for_expression(
            self,
            ExpressionMessage::info("ClearTransformationExpression DataRecord cleared".to_string()),
        );

        Ok(())
    }
}

impl Default for ClearTransformationExpression {
    fn default() -> Self {
        Self::new()
    }
}

impl ClearTransformationExpression {
    pub fn new() -> ClearTransformationExpression {
        Self {
            id: get_next_id(),
            predicate: None,
            hash: OnceCell::new(),
        }
    }

    pub fn new_with_predicate(
        predicate: impl LogicalExpression + 'static,
    ) -> ClearTransformationExpression {
        Self {
            id: get_next_id(),
            predicate: Some(Box::new(predicate)),
            hash: OnceCell::new(),
        }
    }
}
