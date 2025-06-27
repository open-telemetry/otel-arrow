use std::cell::OnceCell;

use crate::{
    data::data_record_resolver::*, error::Error, execution_context::*, expression::*,
    primitives::any_value::AnyValue, value_path::ValuePath,
};

use super::value_expression::*;

#[derive(Debug)]
pub struct ResolveValueExpression {
    id: usize,
    path: ValuePath,
    hash: OnceCell<ExpressionHash>,
}

impl ResolveValueExpression {
    pub fn new(path: &str) -> Result<ResolveValueExpression, Error> {
        Ok(Self {
            id: get_next_id(),
            path: ValuePath::new(path)?,
            hash: OnceCell::new(),
        })
    }

    pub fn get_path(&self) -> &ValuePath {
        &self.path
    }
}

impl Expression for ResolveValueExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"resolve");
                h.add_bytes(b"path:");
                h.add_bytes(self.path.get_raw_value().as_bytes());
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
        output.push_str("resolve ( path: \"");

        output.push_str(self.path.get_raw_value());

        output.push_str("\" )\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl ValueExpressionInternal for ResolveValueExpression {
    fn read_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'a: 'b,
    {
        execution_context.read_any_value(
            self.get_id(),
            self,
            &mut DataRecordAnyValueReadClosureCallback::new(|v| {
                match v {
                    DataRecordReadAnyValueResult::NotFound => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::info(
                                "ResolveValueExpression could not resolve a value".to_string(),
                            ),
                        );
                    }
                    DataRecordReadAnyValueResult::Found(ref any_value) => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::info(format!(
                                "ResolveValueExpression resolved: {any_value:?}"
                            )),
                        );
                    }
                }

                action.invoke_once(v);
            }),
        )
    }
}

impl MutatableValueExpressionInternal for ResolveValueExpression {
    fn set_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
        value: AnyValue,
    ) -> DataRecordSetAnyValueResult
    where
        'a: 'b,
    {
        let result = execution_context.set_any_value(self.get_id(), self, value);

        match &result {
            DataRecordSetAnyValueResult::NotSupported(message) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::warn(format!(
                        "ResolveValueExpression set not supported: {message}"
                    )),
                );
                result
            }
            DataRecordSetAnyValueResult::NotFound => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info("ResolveValueExpression value not found".to_string()),
                );
                result
            }
            DataRecordSetAnyValueResult::Created => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info("ResolveValueExpression created value".to_string()),
                );
                result
            }
            DataRecordSetAnyValueResult::Updated(old_value) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(format!(
                        "ResolveValueExpression replaced value: {old_value:?}"
                    )),
                );
                result
            }
        }
    }

    fn remove_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> DataRecordRemoveAnyValueResult
    where
        'a: 'b,
    {
        let result = execution_context.remove_any_value(self.get_id(), self);

        match &result {
            DataRecordRemoveAnyValueResult::NotFound => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(
                        "ResolveValueExpression could not resolve a value".to_string(),
                    ),
                );
                result
            }
            DataRecordRemoveAnyValueResult::NotSupported(message) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::warn(
                        format!("ResolveValueExpression could not remove value: {message}")
                            .to_string(),
                    ),
                );
                result
            }
            DataRecordRemoveAnyValueResult::Removed(old_value) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(format!(
                        "ResolveValueExpression removed: {old_value:?}"
                    )),
                );
                result
            }
        }
    }
}
