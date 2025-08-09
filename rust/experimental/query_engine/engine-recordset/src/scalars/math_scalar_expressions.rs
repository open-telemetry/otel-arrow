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
