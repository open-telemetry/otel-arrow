use data_engine_expressions::*;

use crate::{execution_context::*, scalars::execute_scalar_expression, *};

pub fn execute_math_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    math_scalar_expression: &'a MathScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let value = match math_scalar_expression {
        MathScalarExpression::Ceiling(u) => {
            let value = execute_scalar_expression(execution_context, u.get_value_expression())?;

            match Value::ceiling(&value.to_value()) {
                Some(i) => {
                    ResolvedValue::Computed(OwnedValue::Integer(IntegerValueStorage::new(i)))
                }
                None => ResolvedValue::Computed(OwnedValue::Null),
            }
        }
        MathScalarExpression::Floor(u) => {
            let value = execute_scalar_expression(execution_context, u.get_value_expression())?;

            match Value::floor(&value.to_value()) {
                Some(i) => {
                    ResolvedValue::Computed(OwnedValue::Integer(IntegerValueStorage::new(i)))
                }
                None => ResolvedValue::Computed(OwnedValue::Null),
            }
        }
    };

    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        math_scalar_expression,
        || format!("Evaluated as: '{value}'"),
    );

    Ok(value)
}
