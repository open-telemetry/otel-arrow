// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;

use crate::{execution_context::*, scalars::execute_scalar_expression, *};

pub fn execute_text_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    text_scalar_expression: &'a TextScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    match text_scalar_expression {
        TextScalarExpression::Replace(r) => {
            let haystack_value =
                execute_scalar_expression(execution_context, r.get_haystack_expression())?;
            let needle_value =
                execute_scalar_expression(execution_context, r.get_needle_expression())?;
            let replacement_value =
                execute_scalar_expression(execution_context, r.get_replacement_expression())?;

            let v = if let Some(result) = Value::replace_matches(
                &haystack_value.to_value(),
                &needle_value.to_value(),
                &replacement_value.to_value(),
                r.get_case_insensitive(),
            ) {
                ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(result)))
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    r,
                    || {
                        format!(
                            "Cannot replace text in '{:?}' haystack using '{:?}' needle and '{:?}' replacement",
                            haystack_value.get_value_type(),
                            needle_value.get_value_type(),
                            replacement_value.get_value_type()
                        )
                    },
                );
                ResolvedValue::Computed(OwnedValue::Null)
            };

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                text_scalar_expression,
                || format!("Evaluated as: '{v}'"),
            );

            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_replace_text_scalar_expression() {
        fn run_test(
            haystack: ScalarExpression,
            needle: ScalarExpression,
            replacement: ScalarExpression,
            expected: Value,
        ) {
            let e = ScalarExpression::Text(TextScalarExpression::Replace(
                ReplaceTextScalarExpression::new(
                    QueryLocation::new_fake(),
                    haystack,
                    needle,
                    replacement,
                    false, // case_insensitive
                ),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let actual = execute_scalar_expression(&execution_context, &e).unwrap();
            assert_eq!(expected, actual.to_value());
        }

        // Basic string replacement
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "A magic trick can turn a cat into a dog",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "cat",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hamster",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "A magic trick can turn a hamster into a dog",
            )),
        );

        // Multiple matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world hello",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi world hi",
            )),
        );

        // No matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "no matches here",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "xyz",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "abc",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "no matches here",
            )),
        );

        // Invalid input type
        run_test(
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "search",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "replace",
            ))),
            Value::Null,
        );
    }

    #[test]
    fn test_execute_replace_text_scalar_expression_with_regex() {
        use regex::Regex;

        fn run_test(
            haystack: ScalarExpression,
            needle: ScalarExpression,
            replacement: ScalarExpression,
            expected: Value,
        ) {
            let e = ScalarExpression::Text(TextScalarExpression::Replace(
                ReplaceTextScalarExpression::new(
                    QueryLocation::new_fake(),
                    haystack,
                    needle,
                    replacement,
                    false, // case_insensitive
                ),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let actual = execute_scalar_expression(&execution_context, &e).unwrap();
            assert_eq!(expected, actual.to_value());
        }

        // Simple regex replacement
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world 123",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(r"\d+").unwrap(),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "XXX",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world XXX",
            )),
        );

        // Regex with capture groups
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "2023-12-25",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap(),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "$3/$2/$1",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "25/12/2023",
            )),
        );

        // Multiple matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "cat cat dog cat",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(r"cat").unwrap(),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hamster",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hamster hamster dog hamster",
            )),
        );

        // Regex with no matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(r"\d+").unwrap(),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "XXX",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
        );
    }

    #[test]
    fn test_execute_replace_text_scalar_expression_case_insensitive() {
        fn run_test(
            haystack: ScalarExpression,
            needle: ScalarExpression,
            replacement: ScalarExpression,
            case_insensitive: bool,
            expected: Value,
        ) {
            let e = ScalarExpression::Text(TextScalarExpression::Replace(
                ReplaceTextScalarExpression::new(
                    QueryLocation::new_fake(),
                    haystack,
                    needle,
                    replacement,
                    case_insensitive,
                ),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let actual = execute_scalar_expression(&execution_context, &e).unwrap();
            assert_eq!(expected, actual.to_value());
        }

        // Case-sensitive replacement (default)
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Hello World",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi",
            ))),
            false,
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Hello World", // No replacement because "hello" != "Hello"
            )),
        );

        // Case-insensitive replacement
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Hello World",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi",
            ))),
            true,
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi World", // "Hello" replaced with "hi"
            )),
        );

        // Case-insensitive with multiple matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "CAT cat Cat",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "cat",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "dog",
            ))),
            true,
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "dog dog dog", // All variants of "cat" replaced
            )),
        );
    }
}
