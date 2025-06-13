use std::cell::OnceCell;

use crate::{
    data::data_record_resolver::*, error::Error, execution_context::ExecutionContext,
    expression::*, logical_expressions::logical_expression::LogicalExpression,
    value_expressions::value_expression::MutatableValueExpression,
};

use super::transformation_expression::TransformationExpressionInternal;

#[derive(Debug)]
pub struct MoveTransformationExpression {
    id: usize,
    destination: Box<dyn MutatableValueExpression>,
    source: Box<dyn MutatableValueExpression>,
    predicate: Option<Box<dyn LogicalExpression>>,
    hash: OnceCell<ExpressionHash>,
}

impl Expression for MoveTransformationExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"move");
                if self.predicate.is_some() {
                    h.add_bytes(b"predicate:");
                    h.add_bytes(self.predicate.as_ref().unwrap().get_hash().get_bytes());
                }
                h.add_bytes(b"destination:");
                h.add_bytes(self.destination.get_hash().get_bytes());
                h.add_bytes(b"source:");
                h.add_bytes(self.source.get_hash().get_bytes());
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
        output.push_str("move (\n");

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

        self.destination
            .write_debug(execution_context, "destination: ", level + 1, output);

        output.push_str(&padding);
        output.push_str(" ,\n");

        self.source
            .write_debug(execution_context, "source: ", level + 1, output);

        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl TransformationExpressionInternal for MoveTransformationExpression {
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
                    "MoveTransformationExpression evaluation skipped".to_string(),
                ),
            );

            return Ok(());
        }

        match self.source.remove_any_value(execution_context) {
            DataRecordRemoveAnyValueResult::NotFound => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::warn(
                        "MoveTransformationExpression AnyValue to write into the destination not found".to_string()));
            }
            DataRecordRemoveAnyValueResult::NotSupported(message) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::warn(
                        format!("MoveTransformationExpression AnyValue to write into the destination could not be mutated: {message}").to_string()));
            }
            DataRecordRemoveAnyValueResult::Removed(any_value) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(
                        "MoveTransformationExpression removed AnyValue from source".to_string(),
                    ),
                );

                match self.destination.set_any_value(execution_context, any_value) {
                    DataRecordSetAnyValueResult::NotSupported(message) => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::warn(
                                format!("MoveTransformationExpression AnyValue destination does not support set operation: {message}")));
                    }
                    DataRecordSetAnyValueResult::NotFound => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::warn(
                                "MoveTransformationExpression AnyValue destination was not found"
                                    .to_string(),
                            ),
                        );
                    }
                    DataRecordSetAnyValueResult::Created => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::info(
                                "MoveTransformationExpression AnyValue created at destination"
                                    .to_string(),
                            ),
                        );
                    }
                    DataRecordSetAnyValueResult::Updated(_) => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::info(
                                "MoveTransformationExpression AnyValue updated in destination"
                                    .to_string(),
                            ),
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

impl MoveTransformationExpression {
    pub fn new(
        destination: impl MutatableValueExpression + 'static,
        source: impl MutatableValueExpression + 'static,
    ) -> MoveTransformationExpression {
        Self {
            id: get_next_id(),
            destination: Box::new(destination),
            source: Box::new(source),
            predicate: None,
            hash: OnceCell::new(),
        }
    }

    pub fn new_with_predicate(
        destination: impl MutatableValueExpression + 'static,
        source: impl MutatableValueExpression + 'static,
        predicate: impl LogicalExpression + 'static,
    ) -> MoveTransformationExpression {
        Self {
            id: get_next_id(),
            destination: Box::new(destination),
            source: Box::new(source),
            predicate: Some(Box::new(predicate)),
            hash: OnceCell::new(),
        }
    }
}
