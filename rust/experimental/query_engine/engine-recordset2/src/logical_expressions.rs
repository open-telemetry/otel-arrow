use data_engine_expressions::*;

use crate::{execution_context::ExecutionContext, *};

pub fn execute_logical_expression<'a, TRecord: Record>(
    _: &ExecutionContext<'a, '_, '_, TRecord>,
    _: &'a LogicalExpression,
) -> Result<bool, ExpressionError> {
    todo!()
}
