use std::{cell::OnceCell, collections::hash_map::Entry};

use crate::{
    data::data_record_resolver::*, error::Error, execution_context::*, expression::*,
    primitives::any_value::AnyValue, value_path::ValuePath,
};

use super::value_expression::*;

#[derive(Debug)]
pub struct VariableValueExpression {
    id: usize,
    name: Box<str>,
    path: Option<ValuePath>,
    hash: OnceCell<ExpressionHash>,
}

impl VariableValueExpression {
    pub fn new(name: &str) -> VariableValueExpression {
        Self {
            id: get_next_id(),
            name: name.into(),
            path: None,
            hash: OnceCell::new(),
        }
    }

    pub fn new_with_path(name: &str, path: &str) -> Result<VariableValueExpression, Error> {
        Ok(Self {
            id: get_next_id(),
            name: name.into(),
            path: Some(ValuePath::new(path)?),
            hash: OnceCell::new(),
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl Expression for VariableValueExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"var");
                h.add_bytes(b"name:");
                h.add_bytes(self.name.as_bytes());
                if self.path.is_some() {
                    h.add_bytes(b"path:");
                    h.add_bytes(self.path.as_ref().unwrap().get_raw_value().as_bytes());
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
        output.push_str("var ( name: \"");

        output.push_str(&self.name);
        output.push('"');

        if self.path.is_some() {
            output.push_str(", path: \"");
            output.push_str(self.path.as_ref().unwrap().get_raw_value());
            output.push('"');
        }

        output.push_str(" )\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl ValueExpressionInternal for VariableValueExpression {
    fn read_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'a: 'b,
    {
        let variables = execution_context.get_variables().borrow();

        let variable = variables.get(self.get_name());

        match variable {
            Some(any_value) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(format!(
                        "VariableValueExpression resolved: {any_value:?}"
                    )),
                );

                if self.path.is_some() {
                    action.invoke_once(self.path.as_ref().unwrap().read(any_value));
                } else {
                    action.invoke_once(DataRecordReadAnyValueResult::Found(any_value));
                }
            }
            None => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(
                        "VariableValueExpression could not resolve value".to_string(),
                    ),
                );
                action.invoke_once(DataRecordReadAnyValueResult::NotFound);
            }
        }
    }
}

impl MutatableValueExpressionInternal for VariableValueExpression {
    fn set_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
        value: AnyValue,
    ) -> DataRecordSetAnyValueResult
    where
        'a: 'b,
    {
        let mut variables = execution_context.get_variables().borrow_mut();

        let variable = variables.entry(self.get_name().to_string());

        match variable {
            Entry::Occupied(mut occupied_entry) => {
                let current_value = occupied_entry.get_mut();

                if self.path.is_some() {
                    return self.path.as_ref().unwrap().set(current_value, value);
                }

                let old_value = current_value.clone();
                *current_value = value;

                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(format!(
                        "VariableValueExpression replaced value: {old_value:?}"
                    )),
                );

                DataRecordSetAnyValueResult::Updated(old_value)
            }
            Entry::Vacant(vacant_entry) => {
                if self.path.is_some() {
                    execution_context.add_message_for_expression(
                        self,
                        ExpressionMessage::warn("Cannot set into a null value".to_string()),
                    );

                    return DataRecordSetAnyValueResult::NotFound;
                }

                vacant_entry.insert(value);

                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info("VariableValueExpression created value".to_string()),
                );

                DataRecordSetAnyValueResult::Created
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
        let mut variables = execution_context.get_variables().borrow_mut();

        if self.path.is_some() {
            match variables.get_mut(self.get_name()) {
                Some(current_value) => {
                    return self.path.as_ref().unwrap().remove(current_value);
                }
                None => {
                    return DataRecordRemoveAnyValueResult::NotFound;
                }
            }
        }

        let variable = variables.remove(self.get_name());

        match variable {
            Some(any_value) => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(format!(
                        "VariableValueExpression removed value: {any_value:?}"
                    )),
                );

                DataRecordRemoveAnyValueResult::Removed(any_value)
            }
            None => {
                execution_context.add_message_for_expression(
                    self,
                    ExpressionMessage::info(
                        "VariableValueExpression could not resolve value".to_string(),
                    ),
                );

                DataRecordRemoveAnyValueResult::NotFound
            }
        }
    }
}
