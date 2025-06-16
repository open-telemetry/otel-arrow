use std::cell::OnceCell;

use crate::{
    data::data_record_resolver::*, error::Error, execution_context::*, expression::*,
    logical_expressions::logical_expression::LogicalExpression,
    value_expressions::value_expression::*,
};

use super::transformation_expression::TransformationExpressionInternal;

#[derive(Debug)]
pub struct RemoveTransformationExpression {
    id: usize,
    target: Box<dyn MutatableValueExpression>,
    predicate: Option<Box<dyn LogicalExpression>>,
    hash: OnceCell<ExpressionHash>,
}

impl Expression for RemoveTransformationExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"remove");
                if self.predicate.is_some() {
                    h.add_bytes(b"predicate:");
                    h.add_bytes(self.predicate.as_ref().unwrap().get_hash().get_bytes());
                }
                h.add_bytes(b"target:");
                h.add_bytes(self.target.get_hash().get_bytes());
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
        output.push_str("remove (\n");

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

        self.target
            .write_debug(execution_context, "target: ", level + 1, output);

        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl TransformationExpressionInternal for RemoveTransformationExpression {
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
                    "RemoveTransformationExpression evaluation skipped".to_string(),
                ),
            );

            return Ok(());
        }

        match self.target.remove_any_value(execution_context) {
            DataRecordRemoveAnyValueResult::NotFound => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(
                        "RemoveTransformationExpression AnyValue not found".to_string(),
                    ),
                );
            }
            DataRecordRemoveAnyValueResult::NotSupported(message) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::warn(
                        format!("RemoveTransformationExpression AnyValue could not be mutated: {message}").to_string()));
            }
            DataRecordRemoveAnyValueResult::Removed(_) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(
                        "RemoveTransformationExpression AnyValue removed from target".to_string(),
                    ),
                );
            }
        }

        Ok(())
    }
}

impl RemoveTransformationExpression {
    pub fn new(target: impl MutatableValueExpression + 'static) -> RemoveTransformationExpression {
        Self {
            id: get_next_id(),
            target: Box::new(target),
            predicate: None,
            hash: OnceCell::new(),
        }
    }

    pub fn new_with_predicate(
        target: impl MutatableValueExpression + 'static,
        predicate: impl LogicalExpression + 'static,
    ) -> RemoveTransformationExpression {
        Self {
            id: get_next_id(),
            target: Box::new(target),
            predicate: Some(Box::new(predicate)),
            hash: OnceCell::new(),
        }
    }
}
