use std::cell::OnceCell;

use crate::{
    data::data_record_resolver::*, error::Error, execution_context::*, expression::*,
    logical_expressions::logical_expression::LogicalExpression, primitives::any_value::AnyValue,
    value_expressions::value_expression::*,
};

use super::transformation_expression::TransformationExpressionInternal;

#[derive(Debug)]
pub struct SetTransformationExpression {
    id: usize,
    destination: Box<dyn MutatableValueExpression>,
    value: Box<dyn ValueExpression>,
    predicate: Option<Box<dyn LogicalExpression>>,
    hash: OnceCell<ExpressionHash>,
}

impl Expression for SetTransformationExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"set");
                if self.predicate.is_some() {
                    h.add_bytes(b"predicate:");
                    h.add_bytes(self.predicate.as_ref().unwrap().get_hash().get_bytes());
                }
                h.add_bytes(b"destination:");
                h.add_bytes(self.destination.get_hash().get_bytes());
                h.add_bytes(b"value:");
                h.add_bytes(self.value.get_hash().get_bytes());
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
        output.push_str("set (\n");

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

        self.value
            .write_debug(execution_context, "value: ", level + 1, output);

        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl TransformationExpressionInternal for SetTransformationExpression {
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
                    "SetTransformationExpression evaluation skipped".to_string(),
                ),
            );

            return Ok(());
        }

        let mut result = Err(Error::ExpressionError(
            self.get_id(),
            Error::NotEvaluated.into(),
        ));

        let mut value: Option<AnyValue> = None;

        self.value.read_any_value(
            execution_context,
            &mut DataRecordAnyValueReadClosureCallback::new(|v| {
                result = Ok(());

                match v {
                    DataRecordReadAnyValueResult::NotFound => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::warn(
                                "SetTransformationExpression AnyValue to write into the destination not found".to_string()));
                    },
                    DataRecordReadAnyValueResult::Found(any_value) => {
                        value = Some(any_value.clone())
                    },
                }
            }));

        if value.is_none() {
            return result;
        }

        match self
            .destination
            .set_any_value(execution_context, value.unwrap())
        {
            DataRecordSetAnyValueResult::NotSupported(message) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::warn(
                        format!("SetTransformationExpression AnyValue destination does not support set operation: {message}")));
            }
            DataRecordSetAnyValueResult::NotFound => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::warn(
                        "SetTransformationExpression AnyValue destination was not found"
                            .to_string(),
                    ),
                );
            }
            DataRecordSetAnyValueResult::Created => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(
                        "SetTransformationExpression AnyValue created at destination".to_string(),
                    ),
                );
            }
            DataRecordSetAnyValueResult::Updated(_) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(
                        "SetTransformationExpression AnyValue updated in destination".to_string(),
                    ),
                );
            }
        }

        Ok(())
    }
}

impl SetTransformationExpression {
    pub fn new(
        destination: impl MutatableValueExpression + 'static,
        value: impl ValueExpression + 'static,
    ) -> SetTransformationExpression {
        Self {
            id: get_next_id(),
            destination: Box::new(destination),
            value: Box::new(value),
            predicate: None,
            hash: OnceCell::new(),
        }
    }

    pub fn new_with_predicate(
        destination: impl MutatableValueExpression + 'static,
        value: impl ValueExpression + 'static,
        predicate: impl LogicalExpression + 'static,
    ) -> SetTransformationExpression {
        Self {
            id: get_next_id(),
            destination: Box::new(destination),
            value: Box::new(value),
            predicate: Some(Box::new(predicate)),
            hash: OnceCell::new(),
        }
    }
}
