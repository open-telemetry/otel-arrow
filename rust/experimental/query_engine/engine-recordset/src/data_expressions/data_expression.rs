use crate::{error::Error, execution_context::ExecutionContext, expression::Expression};

#[allow(private_bounds)]
pub trait DataExpression: DataExpressionInternal {}

impl<T: ?Sized + DataExpressionInternal> DataExpression for T {}

pub(crate) trait DataExpressionInternal: Expression {
    fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<DataExpressionResult, Error>
    where
        'a: 'b;
}

#[derive(Debug)]
pub(crate) enum DataExpressionResult {
    None,
    Drop(usize),
    Include(usize),
}
