use std::cell::OnceCell;

use crate::{
    data::data_record_resolver::*, error::Error, execution_context::*, expression::*,
    primitives::*, value_path::ValuePath,
};

use super::value_expression::*;

#[derive(Debug)]
pub struct ResolveFromAttachedValueExpression {
    id: usize,
    name: StringValueData,
    path: ValuePath,
    hash: OnceCell<ExpressionHash>,
}

impl ResolveFromAttachedValueExpression {
    pub fn new(name: &str, path: &str) -> Result<ResolveFromAttachedValueExpression, Error> {
        Ok(Self {
            id: get_next_id(),
            name: StringValueData::new(name),
            path: ValuePath::new(path)?,
            hash: OnceCell::new(),
        })
    }

    pub fn get_name(&self) -> &str {
        self.name.get_value()
    }

    pub fn get_path(&self) -> &ValuePath {
        &self.path
    }
}

impl Expression for ResolveFromAttachedValueExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"resolve_from_attached");
                h.add_bytes(b"name:");
                h.add_bytes(self.name.get_value().as_bytes());
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
        output.push_str("resolve_from_attached (\n");

        AnyValue::write_debug(&self.name, "name: ", level + 1, output);

        output.push_str(&padding);
        output.push_str(" ,\n");

        output.push_str(&padding);
        output.push_str("\tpath: ");
        output.push_str(self.path.get_raw_value());
        output.push('\n');

        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl ValueExpressionInternal for ResolveFromAttachedValueExpression {
    fn read_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'a: 'b,
    {
        execution_context.read_any_value_from_attached(
            self.get_id(),
            self.name.get_value(),
            &self.path,
            &mut DataRecordAnyValueReadClosureCallback::new(|v| {
                match v {
                    DataRecordReadAnyValueResult::NotFound => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::info(
                                "ResolveFromAttachedValueExpression could not resolve a value"
                                    .to_string(),
                            ),
                        );
                    }
                    DataRecordReadAnyValueResult::Found(any_value) => {
                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::info(format!(
                                "ResolveFromAttachedValueExpression resolved: {any_value:?}"
                            )),
                        );
                    }
                }

                action.invoke_once(v);
            }),
        )
    }
}
