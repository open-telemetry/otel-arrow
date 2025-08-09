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
            execute_unary_operation(execution_context, u, Value::ceiling)?
        }
        MathScalarExpression::Floor(u) => {
            execute_unary_operation(execution_context, u, Value::floor)?
        }
        MathScalarExpression::Add(b) => execute_binary_operation(execution_context, b, Value::add)?,
        MathScalarExpression::Divide(b) => {
            execute_binary_operation(execution_context, b, Value::divide)?
        }
        MathScalarExpression::Multiply(b) => {
            execute_binary_operation(execution_context, b, Value::multiply)?
        }
        MathScalarExpression::Subtract(b) => {
            execute_binary_operation(execution_context, b, Value::subtract)?
        }
    };

    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        math_scalar_expression,
        || format!("Evaluated as: '{value}'"),
    );

    Ok(value)
}

fn execute_unary_operation<'a, 'b, TRecord: Record, F>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    unary_expression: &'a UnaryMathmaticalScalarExpression,
    op: F,
) -> Result<ResolvedValue<'b>, ExpressionError>
where
    F: FnOnce(&Value) -> Option<i64>,
{
    let value =
        execute_scalar_expression(execution_context, unary_expression.get_value_expression())?;

    match (op)(&value.to_value()) {
        Some(i) => Ok(ResolvedValue::Computed(OwnedValue::Integer(
            IntegerValueStorage::new(i),
        ))),
        None => Ok(ResolvedValue::Computed(OwnedValue::Null)),
    }
}

fn execute_binary_operation<'a, 'b, TRecord: Record, F>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    binary_expression: &'a BinaryMathmaticalScalarExpression,
    op: F,
) -> Result<ResolvedValue<'b>, ExpressionError>
where
    F: FnOnce(&Value, &Value) -> Option<NumericValue>,
{
    let left =
        execute_scalar_expression(execution_context, binary_expression.get_left_expression())?;
    let right =
        execute_scalar_expression(execution_context, binary_expression.get_right_expression())?;

    match (op)(&left.to_value(), &right.to_value()) {
        Some(v) => match v {
            NumericValue::Integer(v) => Ok(ResolvedValue::Computed(OwnedValue::Integer(
                IntegerValueStorage::new(v),
            ))),
            NumericValue::Double(v) => Ok(ResolvedValue::Computed(OwnedValue::Double(
                DoubleValueStorage::new(v),
            ))),
        },
        None => Ok(ResolvedValue::Computed(OwnedValue::Null)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_ceiling_and_floor_math_scalar_expression() {
        fn run_test<F>(build: F, input: Vec<(ScalarExpression, Value)>)
        where
            F: Fn(UnaryMathmaticalScalarExpression) -> MathScalarExpression,
        {
            for (inner, expected_value) in input {
                let e = ScalarExpression::Math(build(UnaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    inner,
                )));

                let mut test = TestExecutionContext::new();

                let execution_context = test.create_execution_context();

                let actual_value = execute_scalar_expression(&execution_context, &e).unwrap();
                assert_eq!(expected_value, actual_value.to_value());
            }
        }

        run_test(
            MathScalarExpression::Ceiling,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1),
                    )),
                    Value::Integer(&IntegerValueStorage::new(2)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.1"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(2)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    Value::Null,
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::Null,
                ),
            ],
        );

        run_test(
            MathScalarExpression::Floor,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1),
                    )),
                    Value::Integer(&IntegerValueStorage::new(1)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.1"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(1)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    Value::Null,
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::Null,
                ),
            ],
        );
    }
}
