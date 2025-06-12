use crate::{
    error::Error,
    execution_context::ExecutionContext,
    expression::Expression,
    primitives::any_value::AnyValue,
    value_expressions::value_expression::{self, ValueExpression},
};

#[allow(private_bounds)]
pub trait LogicalExpression: LogicalExpressionInternal {}

impl<T: ?Sized + LogicalExpressionInternal> LogicalExpression for T {}

pub(crate) trait LogicalExpressionInternal: Expression {
    fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<bool, Error>
    where
        'a: 'b;
}

pub(crate) fn compare<'a, 'b>(
    execution_context: &dyn ExecutionContext<'b>,
    expression_id: usize,
    left_value_expr: &'a dyn ValueExpression,
    right_value_expr: &'a dyn ValueExpression,
) -> Result<i32, Error>
where
    'a: 'b,
{
    let mut result = Err(Error::ExpressionError(
        expression_id,
        Error::NotEvaluated.into(),
    ));

    value_expression::resolve_values(
        execution_context,
        left_value_expr,
        right_value_expr,
        |left_value: Option<&AnyValue>, right_value: Option<&AnyValue>| {
            if left_value.is_none() {
                result = Err(Error::new_expression_not_supported(
                    expression_id,
                    "NullValue type on left side of comparison expression is not supported",
                ));
                return;
            }

            if right_value.is_none() {
                let null_value = AnyValue::NullValue;
                result = left_value
                    .unwrap()
                    .compare(execution_context, expression_id, &null_value);
                return;
            }

            result =
                left_value
                    .unwrap()
                    .compare(execution_context, expression_id, right_value.unwrap());
        },
    );

    result
}

pub(crate) fn equals<'a, 'b>(
    execution_context: &dyn ExecutionContext<'b>,
    expression_id: usize,
    left_value_expr: &'a dyn ValueExpression,
    right_value_expr: &'a dyn ValueExpression,
) -> Result<bool, Error>
where
    'a: 'b,
{
    let mut result = Err(Error::ExpressionError(
        expression_id,
        Error::NotEvaluated.into(),
    ));

    value_expression::resolve_values(
        execution_context,
        left_value_expr,
        right_value_expr,
        |left_value: Option<&AnyValue>, right_value: Option<&AnyValue>| {
            let left_is_none = left_value.is_none();
            let right_is_none = right_value.is_none();

            if left_is_none && right_is_none {
                result = Ok(true);
                return;
            }

            if left_is_none {
                let null_value = AnyValue::NullValue;
                result = null_value.equals(execution_context, expression_id, right_value.unwrap());
                return;
            }

            if right_is_none {
                let null_value = AnyValue::NullValue;
                result = left_value
                    .unwrap()
                    .equals(execution_context, expression_id, &null_value);
                return;
            }

            result =
                left_value
                    .unwrap()
                    .equals(execution_context, expression_id, right_value.unwrap());
        },
    );

    result
}
