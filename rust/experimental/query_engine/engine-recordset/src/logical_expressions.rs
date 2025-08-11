use data_engine_expressions::*;

use crate::{execution_context::*, scalars::*, *};

pub fn execute_logical_expression<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    logical_expression: &'a LogicalExpression,
) -> Result<bool, ExpressionError> {
    match logical_expression {
        LogicalExpression::Scalar(s) => {
            let value = execute_scalar_expression(execution_context, s)?;

            if let Some(b) = value.to_value().convert_to_bool() {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    logical_expression,
                    || format!("Evaluated as: '{b}'"),
                );
                Ok(b)
            } else {
                Err(ExpressionError::TypeMismatch(
                    s.get_query_location().clone(),
                    format!(
                        "Value of '{:?}' type returned by scalar expression could not be converted to bool",
                        value.get_value_type()
                    ),
                ))
            }
        }
        LogicalExpression::EqualTo(e) => {
            let left = execute_scalar_expression(execution_context, e.get_left())?;

            let right = execute_scalar_expression(execution_context, e.get_right())?;

            match Value::are_values_equal(
                e.get_query_location(),
                &left.to_value(),
                &right.to_value(),
                e.get_case_insensitive(),
            ) {
                Ok(b) => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        logical_expression,
                        || format!("Evaluated as: '{b}'"),
                    );
                    Ok(b)
                }
                Err(e) => Err(e),
            }
        }
        LogicalExpression::GreaterThan(g) => {
            let left = execute_scalar_expression(execution_context, g.get_left())?;

            let right = execute_scalar_expression(execution_context, g.get_right())?;

            match Value::compare_values(g.get_query_location(), &left.to_value(), &right.to_value())
            {
                Ok(v) => {
                    let r = v > 0;
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        logical_expression,
                        || format!("Evaluated as: '{r}'"),
                    );
                    Ok(r)
                }
                Err(e) => Err(e),
            }
        }
        LogicalExpression::GreaterThanOrEqualTo(g) => {
            let left = execute_scalar_expression(execution_context, g.get_left())?;

            let right = execute_scalar_expression(execution_context, g.get_right())?;

            match Value::compare_values(g.get_query_location(), &left.to_value(), &right.to_value())
            {
                Ok(v) => {
                    let r = v >= 0;
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        logical_expression,
                        || format!("Evaluated as: '{r}'"),
                    );
                    Ok(r)
                }
                Err(e) => Err(e),
            }
        }
        LogicalExpression::Not(n) => {
            match execute_logical_expression(execution_context, n.get_inner_expression()) {
                Ok(mut b) => {
                    b = !b;
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        logical_expression,
                        || format!("Evaluated as: '{b}'"),
                    );
                    Ok(b)
                }
                Err(e) => Err(e),
            }
        }
        LogicalExpression::Chain(c) => {
            let (first, chain) = c.get_expressions();

            let mut result = execute_logical_expression(execution_context, first)?;

            for c in chain {
                match c {
                    ChainedLogicalExpression::Or(or) => {
                        if result {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                or,
                                || {
                                    "Short-circuiting chain because left-hand side of OR is true"
                                        .into()
                                },
                            );
                            break;
                        }

                        result = execute_logical_expression(execution_context, or)?;
                    }
                    ChainedLogicalExpression::And(and) => {
                        if !result {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                and,
                                || {
                                    "Short-circuiting chain because left-hand side of AND is false"
                                        .into()
                                },
                            );
                            break;
                        }

                        result = execute_logical_expression(execution_context, and)?;
                    }
                }
            }

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                logical_expression,
                || format!("Evaluated as: '{result}'"),
            );

            Ok(result)
        }
        LogicalExpression::Contains(c) => {
            let haystack = execute_scalar_expression(execution_context, c.get_haystack())?;
            let needle = execute_scalar_expression(execution_context, c.get_needle())?;

            match Value::contains(
                c.get_query_location(),
                &haystack.to_value(),
                &needle.to_value(),
                c.get_case_insensitive(),
            ) {
                Ok(b) => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        logical_expression,
                        || format!("Evaluated as: '{b}'"),
                    );
                    Ok(b)
                }
                Err(e) => Err(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_scalar_logical_expression() {
        let run_test = |logical_expression, expected_value: bool| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let value =
                execute_logical_expression(&execution_context, &logical_expression).unwrap();

            assert_eq!(expected_value, value);
        };

        run_test(
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
            true,
        );

        run_test(
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            ))),
            false,
        );
    }

    #[test]
    fn test_execute_equal_to_logical_expression() {
        let run_test = |logical_expression, expected_value: bool| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let value =
                execute_logical_expression(&execution_context, &logical_expression).unwrap();

            assert_eq!(expected_value, value);
        };

        run_test(
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                )),
                ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                )),
                false,
            )),
            true,
        );

        run_test(
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "VALUE"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "value"),
                )),
                false,
            )),
            false,
        );

        // Test case-insensitive string equality
        run_test(
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "VALUE"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "value"),
                )),
                true,
            )),
            true,
        );
    }

    #[test]
    fn test_execute_greater_than_logical_expression() {
        let run_test = |logical_expression, expected_value: bool| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let value =
                execute_logical_expression(&execution_context, &logical_expression).unwrap();

            assert_eq!(expected_value, value);
        };

        run_test(
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 17),
                )),
            )),
            true,
        );

        run_test(
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
            )),
            false,
        );
    }

    #[test]
    fn test_execute_greater_than_or_equal_to_logical_expression() {
        let run_test = |logical_expression, expected_value: bool| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let value =
                execute_logical_expression(&execution_context, &logical_expression).unwrap();

            assert_eq!(expected_value, value);
        };

        run_test(
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 17),
                )),
            )),
            true,
        );

        run_test(
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
            )),
            true,
        );

        run_test(
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 19),
                )),
            )),
            false,
        );
    }

    #[test]
    fn test_execute_not_logical_expression() {
        let run_test = |logical_expression, expected_value: bool| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let value =
                execute_logical_expression(&execution_context, &logical_expression).unwrap();

            assert_eq!(expected_value, value);
        };

        run_test(
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
            )),
            true,
        );

        run_test(
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
            )),
            false,
        );
    }

    #[test]
    fn test_execute_chain_logical_expression() {
        let run_test = |logical_expression, expected_value: bool| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let value =
                execute_logical_expression(&execution_context, &logical_expression).unwrap();

            assert_eq!(expected_value, value);
        };

        // Test: true
        run_test(
            LogicalExpression::Chain(ChainLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
            )),
            true,
        );

        // Test: false
        run_test(
            LogicalExpression::Chain(ChainLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
            )),
            false,
        );

        // Test: true || false
        let mut chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
        );

        chain.push_or(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                false,
            )),
        )));

        run_test(LogicalExpression::Chain(chain), true);

        // Test: false || true
        let mut chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            ))),
        );

        chain.push_or(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
        )));

        run_test(LogicalExpression::Chain(chain), true);

        // Test: false && true
        let mut chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            ))),
        );

        chain.push_and(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
        )));

        run_test(LogicalExpression::Chain(chain), false);

        // Test: true && false
        let mut chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
        );

        chain.push_and(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                false,
            )),
        )));

        run_test(LogicalExpression::Chain(chain), false);

        // Test: true && true
        let mut chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
        );

        chain.push_and(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
        )));

        run_test(LogicalExpression::Chain(chain), true);
    }

    #[test]
    fn test_execute_contains_logical_expression() {
        let run_test = |logical_expression, expected_value: bool| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let value =
                execute_logical_expression(&execution_context, &logical_expression).unwrap();

            assert_eq!(expected_value, value);
        };

        // Test array contains string
        run_test(
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Array(
                    ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "hello",
                            )),
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "world",
                            )),
                        ],
                    ),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                )),
                false,
            )),
            true,
        );

        // Test array does not contain string
        run_test(
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Array(
                    ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "hello",
                            )),
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "world",
                            )),
                        ],
                    ),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "foo"),
                )),
                false,
            )),
            false,
        );

        // Test string contains string
        run_test(
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "world"),
                )),
                false,
            )),
            true,
        );

        // Test string contains string case insensitive
        run_test(
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "WORLD"),
                )),
                true,
            )),
            true,
        );

        // Test string does not contain string
        run_test(
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "foo"),
                )),
                false,
            )),
            false,
        );
    }
}
