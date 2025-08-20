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
        MathScalarExpression::Add(b) => execute_binary_operation(execution_context, b, Value::add)?,
        MathScalarExpression::Bin(b) => execute_binary_operation(execution_context, b, Value::bin)?,
        MathScalarExpression::Ceiling(u) => execute_unary_operation(execution_context, u, |v| {
            Value::ceiling(v).map(NumericValue::Integer)
        })?,
        MathScalarExpression::Divide(b) => {
            execute_binary_operation(execution_context, b, Value::divide)?
        }
        MathScalarExpression::Floor(u) => execute_unary_operation(execution_context, u, |v| {
            Value::floor(v).map(NumericValue::Integer)
        })?,
        MathScalarExpression::Modulus(b) => {
            execute_binary_operation(execution_context, b, Value::modulus)?
        }
        MathScalarExpression::Multiply(b) => {
            execute_binary_operation(execution_context, b, Value::multiply)?
        }
        MathScalarExpression::Negate(n) => {
            execute_unary_operation(execution_context, n, Value::negate)?
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
    F: FnOnce(&Value) -> Option<NumericValue>,
{
    let value =
        execute_scalar_expression(execution_context, unary_expression.get_value_expression())?;

    match (op)(&value.to_value()) {
        Some(v) => Ok(numeric_value_to_resolved_value(v)),
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
        Some(v) => Ok(numeric_value_to_resolved_value(v)),
        None => Ok(ResolvedValue::Computed(OwnedValue::Null)),
    }
}

fn numeric_value_to_resolved_value<'a>(value: NumericValue) -> ResolvedValue<'a> {
    match value {
        NumericValue::Integer(i) => {
            ResolvedValue::Computed(OwnedValue::Integer(IntegerValueStorage::new(i)))
        }
        NumericValue::DateTime(d) => {
            ResolvedValue::Computed(OwnedValue::DateTime(DateTimeValueStorage::new(d)))
        }
        NumericValue::Double(d) => {
            ResolvedValue::Computed(OwnedValue::Double(DoubleValueStorage::new(d)))
        }
        NumericValue::TimeSpan(t) => {
            ResolvedValue::Computed(OwnedValue::TimeSpan(TimeSpanValueStorage::new(t)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_ceiling_floor_negate_math_scalar_expression() {
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

        run_test(
            MathScalarExpression::Negate,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1),
                    )),
                    Value::Double(&DoubleValueStorage::new(-1.1)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    Value::Integer(&IntegerValueStorage::new(-1)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.1"),
                    )),
                    Value::Double(&DoubleValueStorage::new(-1.1)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(-1)),
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
                (
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    Value::Integer(&IntegerValueStorage::new(-1)),
                ),
            ],
        );
    }

    #[test]
    fn test_execute_add_subtract_multiply_divide_math_scalar_expression() {
        fn run_test<F>(build: F, input: Vec<(ScalarExpression, ScalarExpression, Value)>)
        where
            F: Fn(BinaryMathmaticalScalarExpression) -> MathScalarExpression,
        {
            for (left, right, expected_value) in input {
                let e = ScalarExpression::Math(build(BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    left,
                    right,
                )));

                let mut test = TestExecutionContext::new();

                let execution_context = test.create_execution_context();

                let actual_value = execute_scalar_expression(&execution_context, &e).unwrap();
                assert_eq!(expected_value, actual_value.to_value());
            }
        }

        run_test(
            MathScalarExpression::Add,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.01),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.18),
                    )),
                    Value::Double(&DoubleValueStorage::new(2.19)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Value::Integer(&IntegerValueStorage::new(19)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0),
                    )),
                    Value::Double(&DoubleValueStorage::new(2.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.01"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    Value::Double(&DoubleValueStorage::new(2.19)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "18"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(19)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.0"),
                    )),
                    Value::Null,
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::Null,
                ),
            ],
        );

        run_test(
            MathScalarExpression::Subtract,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.01),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.18),
                    )),
                    Value::Double(&DoubleValueStorage::new(-0.16999999999999993)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Value::Integer(&IntegerValueStorage::new(-17)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0),
                    )),
                    Value::Double(&DoubleValueStorage::new(0.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.01"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    Value::Double(&DoubleValueStorage::new(-0.16999999999999993)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "18"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(-17)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.0"),
                    )),
                    Value::Null,
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::Null,
                ),
            ],
        );

        run_test(
            MathScalarExpression::Multiply,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.01),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.18),
                    )),
                    Value::Double(&DoubleValueStorage::new(1.1918)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Value::Integer(&IntegerValueStorage::new(18)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0),
                    )),
                    Value::Double(&DoubleValueStorage::new(1.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.01"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    Value::Double(&DoubleValueStorage::new(1.1918)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "18"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(18)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.0"),
                    )),
                    Value::Null,
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::Null,
                ),
            ],
        );

        run_test(
            MathScalarExpression::Divide,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.01),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.18),
                    )),
                    Value::Double(&DoubleValueStorage::new(0.8559322033898306)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Value::Integer(&IntegerValueStorage::new(0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0),
                    )),
                    Value::Double(&DoubleValueStorage::new(1.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.01"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    Value::Double(&DoubleValueStorage::new(0.8559322033898306)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "18"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.0"),
                    )),
                    Value::Null,
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::Null,
                ),
            ],
        );
    }

    #[test]
    fn test_execute_modulus_and_bin_math_scalar_expression() {
        fn run_test<F>(build: F, input: Vec<(ScalarExpression, ScalarExpression, Value)>)
        where
            F: Fn(BinaryMathmaticalScalarExpression) -> MathScalarExpression,
        {
            for (left, right, expected_value) in input {
                let e = ScalarExpression::Math(build(BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    left,
                    right,
                )));

                let mut test = TestExecutionContext::new();

                let execution_context = test.create_execution_context();

                let actual_value = execute_scalar_expression(&execution_context, &e).unwrap();
                assert_eq!(expected_value, actual_value.to_value());
            }
        }

        run_test(
            MathScalarExpression::Modulus,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 10.18),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 3.0),
                    )),
                    Value::Double(&DoubleValueStorage::new(1.1799999999999997)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                    )),
                    Value::Integer(&IntegerValueStorage::new(1)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 3.0),
                    )),
                    Value::Double(&DoubleValueStorage::new(1.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "10.18"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "3.0"),
                    )),
                    Value::Double(&DoubleValueStorage::new(1.1799999999999997)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "10"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "3"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(1)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.0"),
                    )),
                    Value::Null,
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::Null,
                ),
            ],
        );

        run_test(
            MathScalarExpression::Bin,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 10018.18),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 100.0),
                    )),
                    Value::Double(&DoubleValueStorage::new(10000.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10018),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 100),
                    )),
                    Value::Integer(&IntegerValueStorage::new(10000)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10018),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 100.0),
                    )),
                    Value::Double(&DoubleValueStorage::new(10000.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "10018.18"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "100.0"),
                    )),
                    Value::Double(&DoubleValueStorage::new(10000.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "10018"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "100"),
                    )),
                    Value::Integer(&IntegerValueStorage::new(10000)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.0"),
                    )),
                    Value::Null,
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::Null,
                ),
            ],
        );
    }
}
