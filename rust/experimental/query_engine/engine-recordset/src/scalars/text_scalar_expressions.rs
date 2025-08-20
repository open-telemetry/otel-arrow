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
        TextScalarExpression::Match(m) => {
            // Execute the input expression to get the string to match
            let input_value =
                execute_scalar_expression(execution_context, m.get_input_expression())?;
            let input_string = match input_value.to_value() {
                Value::String(s) => s.get_value(),
                _ => {
                    return Err(ExpressionError::ValidationFailure(
                        m.get_query_location().clone(),
                        format!(
                            "Text match input must be a string, got: '{:?}'",
                            input_value.get_value_type()
                        ),
                    ));
                }
            };

            // Execute the pattern expression to get the regex
            let pattern_value = execute_scalar_expression(execution_context, m.get_pattern())?;
            let pattern_string = match pattern_value.to_value() {
                Value::String(s) => s.get_value(),
                _ => {
                    return Err(ExpressionError::ValidationFailure(
                        m.get_query_location().clone(),
                        format!(
                            "Text match pattern must be a string, got: '{:?}'",
                            pattern_value.get_value_type()
                        ),
                    ));
                }
            };

            // Compile the regex pattern
            let regex = match regex::Regex::new(pattern_string) {
                Ok(regex) => regex,
                Err(e) => {
                    return Err(ExpressionError::ValidationFailure(
                        m.get_query_location().clone(),
                        format!("Invalid regex pattern '{}': {}", pattern_string, e),
                    ));
                }
            };

            // Check if the regex matches the input string
            let matches = regex.is_match(input_string);

            let v = ResolvedValue::Computed(OwnedValue::Boolean(BooleanValueStorage::new(matches)));

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                text_scalar_expression,
                || format!("Evaluated as: '{v}'"),
            );

            Ok(v)
        }
        TextScalarExpression::Extract(e) => {
            // Execute the input expression to get the string to parse
            let input_value =
                execute_scalar_expression(execution_context, e.get_input_expression())?;
            let input_string = match input_value.to_value() {
                Value::String(s) => s.get_value(),
                _ => {
                    return Err(ExpressionError::ValidationFailure(
                        e.get_query_location().clone(),
                        format!(
                            "Text extract input must be a string, got: '{:?}'",
                            input_value.get_value_type()
                        ),
                    ));
                }
            };

            // Execute the pattern expression to get the regex
            let pattern_value =
                execute_scalar_expression(execution_context, e.get_pattern_expression())?;
            let pattern_string = match pattern_value.to_value() {
                Value::String(s) => s.get_value(),
                _ => {
                    return Err(ExpressionError::ValidationFailure(
                        e.get_query_location().clone(),
                        format!(
                            "Text extract pattern must be a string, got: '{:?}'",
                            pattern_value.get_value_type()
                        ),
                    ));
                }
            };

            // Execute the capture name or index expression
            let capture_value =
                execute_scalar_expression(execution_context, e.get_capture_name_or_index())?;

            // Compile the regex pattern
            let regex = match regex::Regex::new(pattern_string) {
                Ok(regex) => regex,
                Err(err) => {
                    return Err(ExpressionError::ValidationFailure(
                        e.get_query_location().clone(),
                        format!("Invalid regex pattern '{}': {}", pattern_string, err),
                    ));
                }
            };

            // Apply the regex to the input string and extract the capture group
            let v = if let Some(captures) = regex.captures(input_string) {
                match capture_value.to_value() {
                    Value::String(s) => {
                        // Named capture group
                        let capture_name = s.get_value();
                        if let Some(matched) = captures.name(capture_name) {
                            ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(
                                matched.as_str().into(),
                            )))
                        } else {
                            // Named capture group not found - return empty string
                            ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(
                                "".into(),
                            )))
                        }
                    }
                    Value::Integer(i) => {
                        // Indexed capture group
                        let index = i.get_value() as usize;
                        if let Some(matched) = captures.get(index) {
                            ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(
                                matched.as_str().into(),
                            )))
                        } else {
                            // Indexed capture group not found - return empty string
                            ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(
                                "".into(),
                            )))
                        }
                    }
                    _ => {
                        return Err(ExpressionError::ValidationFailure(
                            e.get_query_location().clone(),
                            format!(
                                "Text extract capture name or index must be a string or integer, got: '{:?}'",
                                capture_value.get_value_type()
                            ),
                        ));
                    }
                }
            } else {
                // No match found - return empty string
                ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new("".into())))
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

    #[test]
    fn test_execute_extract_text_scalar_expression() {
        fn run_test(
            input: ScalarExpression,
            pattern: ScalarExpression,
            capture: ScalarExpression,
            expected: Value,
        ) {
            let e = ScalarExpression::Text(TextScalarExpression::Extract(
                ExtractTextScalarExpression::new(
                    QueryLocation::new_fake(),
                    input,
                    pattern,
                    capture,
                ),
            ));

            let mut test = TestExecutionContext::new();
            let execution_context = test.create_execution_context();
            let result = execute_scalar_expression(&execution_context, &e).unwrap();

            assert_eq!(result.to_value(), expected);
        }

        // Test named capture group extraction
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "John Smith age 30",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                r"(?P<name>\w+) (?P<lastname>\w+) age (?P<age>\d+)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "name",
            ))),
            Value::String(&StringValueStorage::new("John".into())),
        );

        // Test another named capture group
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "John Smith age 30",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                r"(?P<name>\w+) (?P<lastname>\w+) age (?P<age>\d+)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "age",
            ))),
            Value::String(&StringValueStorage::new("30".into())),
        );

        // Test indexed capture group (1 = first capture group)
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "John Smith age 30",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                r"(\w+) (\w+) age (\d+)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
            Value::String(&StringValueStorage::new("John".into())),
        );

        // Test indexed capture group (3 = third capture group)
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "John Smith age 30",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                r"(\w+) (\w+) age (\d+)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
            )),
            Value::String(&StringValueStorage::new("30".into())),
        );

        // Test non-existent named capture group - should return empty string
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "John Smith age 30",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                r"(?P<name>\w+) (?P<lastname>\w+) age (?P<age>\d+)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "nonexistent",
            ))),
            Value::String(&StringValueStorage::new("".into())),
        );

        // Test non-existent indexed capture group - should return empty string
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "John Smith age 30",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                r"(\w+) (\w+) age (\d+)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    5, // Index 5 doesn't exist
                ),
            )),
            Value::String(&StringValueStorage::new("".into())),
        );

        // Test no match - should return empty string
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "This doesn't match",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                r"(\w+) (\w+) age (\d+)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
            Value::String(&StringValueStorage::new("".into())),
        );
    }

    #[test]
    fn test_execute_match_text_scalar_expression() {
        fn run_test(input: ScalarExpression, pattern: ScalarExpression, expected: bool) {
            let e = ScalarExpression::Text(TextScalarExpression::Match(
                MatchTextScalarExpression::new(QueryLocation::new_fake(), input, pattern),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let actual = execute_scalar_expression(&execution_context, &e).unwrap();
            assert_eq!(
                Value::Boolean(&BooleanValueStorage::new(expected)),
                actual.to_value()
            );
        }

        // Basic regex match - should match
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: NotifySliceRelease",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: .*",
            ))),
            true,
        );

        // Basic regex match - should not match
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Warning: Something happened",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: .*",
            ))),
            false,
        );

        // Case sensitive match
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "^[a-z ]+$",
            ))),
            true,
        );

        // Case sensitive match - should fail
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Hello World",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "^[a-z ]+$",
            ))),
            false,
        );

        // Complex regex with character classes
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "user123@example.com",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                r"^[a-zA-Z0-9]+@[a-zA-Z0-9]+\.[a-zA-Z]{2,}$",
            ))),
            true,
        );

        // Empty string match
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "^$",
            ))),
            true,
        );

        // No match
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "This doesn't match anything",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "^\\d+$",
            ))),
            false,
        );
    }
}
