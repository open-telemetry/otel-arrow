use crate::{error::Error, execution_context::ExecutionContext, expression::Expression};

#[allow(private_bounds)]
pub trait TransformationExpression: TransformationExpressionInternal {}

impl<T: ?Sized + TransformationExpressionInternal> TransformationExpression for T {}

pub(crate) trait TransformationExpressionInternal: Expression {
    fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<(), Error>
    where
        'a: 'b;
}
