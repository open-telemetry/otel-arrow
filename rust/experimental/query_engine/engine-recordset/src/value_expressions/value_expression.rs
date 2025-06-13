use crate::{
    data::data_record_resolver::*, execution_context::*, expression::Expression,
    primitives::any_value::AnyValue,
};

#[allow(private_bounds)]
pub trait ValueExpression: ValueExpressionInternal {}

impl<T: ?Sized + ValueExpressionInternal> ValueExpression for T {}

pub(crate) trait ValueExpressionInternal: Expression {
    fn read_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'a: 'b;
}

#[allow(private_bounds)]
pub trait MutatableValueExpression: MutatableValueExpressionInternal + ValueExpression {}

impl<T: ?Sized + MutatableValueExpressionInternal> MutatableValueExpression for T {}

pub(crate) trait MutatableValueExpressionInternal: ValueExpressionInternal {
    fn set_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
        value: AnyValue,
    ) -> DataRecordSetAnyValueResult
    where
        'a: 'b;

    fn remove_any_value<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> DataRecordRemoveAnyValueResult
    where
        'a: 'b;
}

pub(crate) fn resolve_values<'a, 'b, F>(
    execution_context: &dyn ExecutionContext<'b>,
    value1_expr: &'a dyn ValueExpression,
    value2_expr: &'a dyn ValueExpression,
    action: F,
) where
    'a: 'b,
    F: FnOnce(Option<&AnyValue>, Option<&AnyValue>),
{
    value1_expr.read_any_value(
        execution_context,
        &mut DataRecordAnyValueReadClosureCallback::new(|v| {
            match v {
                DataRecordReadAnyValueResult::NotFound => {
                    complete(execution_context, None, value2_expr, action)
                }
                DataRecordReadAnyValueResult::Found(any_value) => {
                    complete(execution_context, Some(any_value), value2_expr, action)
                }
            };
        }),
    );

    fn complete<'a, 'b, F>(
        execution_context: &dyn ExecutionContext<'b>,
        value1: Option<&AnyValue>,
        value2_expr: &'a dyn ValueExpression,
        action: F,
    ) where
        'a: 'b,
        F: FnOnce(Option<&AnyValue>, Option<&AnyValue>),
    {
        value2_expr.read_any_value(
            execution_context,
            &mut DataRecordAnyValueReadClosureCallback::new(|v| {
                match v {
                    DataRecordReadAnyValueResult::NotFound => action(value1, None),
                    DataRecordReadAnyValueResult::Found(any_value) => {
                        action(value1, Some(any_value))
                    }
                };
            }),
        );
    }
}
