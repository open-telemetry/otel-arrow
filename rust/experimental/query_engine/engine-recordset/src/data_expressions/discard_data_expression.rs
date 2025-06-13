use std::cell::OnceCell;

use crate::{
    error::Error, execution_context::ExecutionContext, expression::*,
    logical_expressions::logical_expression::LogicalExpression,
};

use super::data_expression::{DataExpressionInternal, DataExpressionResult};

#[derive(Debug)]
pub struct DiscardDataExpression {
    id: usize,
    predicate: Option<Box<dyn LogicalExpression>>,
    hash: OnceCell<ExpressionHash>,
}

impl Expression for DiscardDataExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"discard");
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
        output.push_str("discard (\n");

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

impl DataExpressionInternal for DiscardDataExpression {
    fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<DataExpressionResult, Error>
    where
        'a: 'b,
    {
        let summary_index = execution_context.get_summary_index();
        if summary_index.is_some() {
            execution_context.add_message_for_expression(
                self,
                ExpressionMessage::info("DiscardDataExpression evaluation skipped because the current record has been included in a summary".to_string()));

            return Ok(DataExpressionResult::None);
        }

        if self.predicate.is_some() {
            let result = self
                .predicate
                .as_ref()
                .unwrap()
                .evaluate(execution_context)?;
            if !result {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info("DiscardDataExpression evaluation skipped".to_string()),
                );

                return Ok(DataExpressionResult::None);
            }
        }

        execution_context.add_message_for_expression(
            self,
            ExpressionMessage::info("DiscardDataExpression evaluated".to_string()),
        );

        Ok(DataExpressionResult::Drop(self.get_id()))
    }
}

impl Default for DiscardDataExpression {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscardDataExpression {
    pub fn new() -> DiscardDataExpression {
        Self {
            id: get_next_id(),
            predicate: None,
            hash: OnceCell::new(),
        }
    }

    pub fn new_with_predicate(
        predicate: impl LogicalExpression + 'static,
    ) -> DiscardDataExpression {
        Self {
            id: get_next_id(),
            predicate: Some(Box::new(predicate)),
            hash: OnceCell::new(),
        }
    }
}
