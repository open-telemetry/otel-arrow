use std::cell::OnceCell;

use crate::{
    data::data_record_resolver::*, execution_context::*, expression::*,
    primitives::any_value::AnyValue,
};

use super::value_expression::*;

#[derive(Debug)]
pub struct StaticValueExpression {
    id: usize,
    value: AnyValue,
    hash: OnceCell<ExpressionHash>,
}

impl StaticValueExpression {
    pub fn new(value: AnyValue) -> StaticValueExpression {
        Self {
            id: get_next_id(),
            value,
            hash: OnceCell::new(),
        }
    }
}

impl Expression for StaticValueExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"static");
                h.add_bytes(b"value:");
                AnyValue::add_hash_bytes(&self.value, h);
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
        output.push_str("static ( value: ");

        output.push_str(format!("{:?}", self.value).as_str());

        output.push_str(" )\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl ValueExpressionInternal for StaticValueExpression {
    fn read_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'a: 'b,
    {
        let value = &self.value;

        execution_context.add_message_for_expression(
            self,
            ExpressionMessage::info(format!("StaticValueExpression resolved: {:?}", value)),
        );

        action.invoke_once(DataRecordReadAnyValueResult::Found(value));
    }
}
