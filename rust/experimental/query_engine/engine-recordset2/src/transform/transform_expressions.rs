use data_engine_expressions::*;

use crate::{execution_context::ExecutionContext, *};

pub fn execute_transform_expression<'a, TRecord: Record>(
    _: &ExecutionContext<'a, '_, '_, TRecord>,
    _: &'a TransformExpression,
) -> Result<(), ExpressionError> {
    todo!()
}
