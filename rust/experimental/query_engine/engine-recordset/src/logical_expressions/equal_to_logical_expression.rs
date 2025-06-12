use std::cell::OnceCell;

use crate::{
    error::Error, execution_context::ExecutionContext, expression::*,
    value_expressions::value_expression::ValueExpression,
};

use super::logical_expression::{LogicalExpressionInternal, equals};

#[derive(Debug)]
pub struct EqualToLogicalExpression {
    id: usize,
    left: Box<dyn ValueExpression>,
    right: Box<dyn ValueExpression>,
    hash: OnceCell<ExpressionHash>,
}

impl EqualToLogicalExpression {
    pub fn new(
        left: impl ValueExpression + 'static,
        right: impl ValueExpression + 'static,
    ) -> EqualToLogicalExpression {
        Self {
            id: get_next_id(),
            left: Box::new(left),
            right: Box::new(right),
            hash: OnceCell::new(),
        }
    }
}

impl Expression for EqualToLogicalExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"equal_to");
                h.add_bytes(b"left:");
                h.add_bytes(self.left.get_hash().get_bytes());
                h.add_bytes(b"right:");
                h.add_bytes(self.right.get_hash().get_bytes());
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
        output.push_str("equal_to (\n");

        self.left
            .write_debug(execution_context, "left: ", level + 1, output);
        output.push_str(&padding);
        output.push_str(" ,\n");

        self.right
            .write_debug(execution_context, "right: ", level + 1, output);
        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl LogicalExpressionInternal for EqualToLogicalExpression {
    fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<bool, Error>
    where
        'a: 'b,
    {
        let equal = equals(
            execution_context,
            self.get_id(),
            self.left.as_ref(),
            self.right.as_ref(),
        )?;

        execution_context.add_message_for_expression(
            self,
            ExpressionMessage::info(
                either!(equal => "EqualToLogicalExpression evaluated as true"; "EqualToLogicalExpression evaluated as false").to_string()));

        Ok(equal)
    }
}
