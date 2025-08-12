use data_engine_expressions::*;

use crate::{execution_context::*, scalars::execute_scalar_expression, *};

pub fn execute_parse_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    parse_scalar_expression: &'a ParseScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let v = match parse_scalar_expression {
        ParseScalarExpression::Json(p) => {
            let inner_value =
                execute_scalar_expression(execution_context, p.get_inner_expression())?;

            let value = inner_value.to_value();

            match value {
                Value::String(s) => ResolvedValue::Computed(OwnedValue::from_json(
                    p.get_query_location(),
                    s.get_value(),
                )?),
                _ => {
                    return Err(ExpressionError::ParseError(
                        p.get_query_location().clone(),
                        format!(
                            "Input of '{:?}' type could not be pased as JSON",
                            value.get_value_type()
                        ),
                    ));
                }
            }
        }
        ParseScalarExpression::Regex(p) => {
            let pattern_value = execute_scalar_expression(execution_context, p.get_pattern())?;

            let options_value = if let Some(o) = p.get_options() {
                Some(execute_scalar_expression(execution_context, o)?)
            } else {
                None
            };

            let regex = match (pattern_value, options_value) {
                (pattern, None) => {
                    Value::parse_regex(p.get_query_location(), &pattern.to_value(), None)?
                }
                (pattern, Some(options)) => Value::parse_regex(
                    p.get_query_location(),
                    &pattern.to_value(),
                    Some(&options.to_value()),
                )?,
            };

            ResolvedValue::Computed(OwnedValue::Regex(RegexValueStorage::new(regex)))
        }
    };

    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        parse_scalar_expression,
        || format!("Evaluated as: {v}"),
    );

    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_execute_parse_json_scalar_expression() {
        fn run_test_success(input: &str, expected_value: Value) {
            let expression = ScalarExpression::Parse(ParseScalarExpression::Json(
                ParseJsonScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), input),
                    )),
                ),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let actual_value = execute_scalar_expression(&execution_context, &expression).unwrap();
            assert_eq!(expected_value, actual_value.to_value());
        }

        fn run_test_failure(input: &str) {
            let expression = ScalarExpression::Parse(ParseScalarExpression::Json(
                ParseJsonScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), input),
                    )),
                ),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let actual_value =
                execute_scalar_expression(&execution_context, &expression).unwrap_err();
            assert!(matches!(actual_value, ExpressionError::ParseError(_, _)));
        }

        run_test_success(
            "18",
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
        );
        run_test_failure("hello world");
    }

    #[test]
    pub fn test_execute_parse_regex_scalar_expression() {
        fn run_test_success(pattern: ScalarExpression, options: Option<ScalarExpression>) {
            let expression = ScalarExpression::Parse(ParseScalarExpression::Regex(
                ParseRegexScalarExpression::new(QueryLocation::new_fake(), pattern, options),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let actual_value = execute_scalar_expression(&execution_context, &expression).unwrap();
            assert_eq!(ValueType::Regex, actual_value.get_value_type());
        }

        fn run_test_failure(pattern: ScalarExpression, options: Option<ScalarExpression>) {
            let expression = ScalarExpression::Parse(ParseScalarExpression::Regex(
                ParseRegexScalarExpression::new(QueryLocation::new_fake(), pattern, options),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            execute_scalar_expression(&execution_context, &expression).unwrap_err();
        }

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                ".*",
            ))),
            None,
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                ".*",
            ))),
            Some(ScalarExpression::Static(StaticScalarExpression::String(
                StringScalarExpression::new(QueryLocation::new_fake(), "i"),
            ))),
        );

        run_test_failure(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "(",
            ))),
            None,
        );

        run_test_failure(
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
            None,
        );

        run_test_failure(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                ".*",
            ))),
            Some(ScalarExpression::Static(StaticScalarExpression::Null(
                NullScalarExpression::new(QueryLocation::new_fake()),
            ))),
        );
    }
}
