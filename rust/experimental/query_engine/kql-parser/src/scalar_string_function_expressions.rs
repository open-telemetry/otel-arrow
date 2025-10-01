// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_string_unary_expressions(
    string_unary_expressions_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let rule = string_unary_expressions_rule.into_inner().next().unwrap();

    match rule.as_rule() {
        Rule::strlen_expression => parse_strlen_expression(rule, scope),
        Rule::replace_string_expression => parse_replace_string_expression(rule, scope),
        Rule::substring_expression => parse_substring_expression(rule, scope),
        Rule::strcat_expression => parse_strcat_expression(rule, scope),
        Rule::strcat_delim_expression => parse_strcat_delim_expression(rule, scope),
        Rule::extract_expression => parse_extract_expression(rule, scope),
        _ => panic!("Unexpected rule in string_unary_expressions_rule: {rule}"),
    }
}

fn parse_strlen_expression(
    strlen_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&strlen_expression_rule);

    let mut strlen_rules = strlen_expression_rule.into_inner();

    let inner_expression_rule = strlen_rules.next().unwrap();
    let inner_expression_rule_location = to_query_location(&inner_expression_rule);
    let mut inner_expression = parse_scalar_expression(inner_expression_rule, scope)?;

    if let Some(v) = scope.try_resolve_value_type(&mut inner_expression)?
        && v != ValueType::String
    {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: inner_expression_rule_location,
            diagnostic_id: "KS141",
            message: "The expression must have one of the types: string or dynamic".into(),
        });
    }

    Ok(ScalarExpression::Length(LengthScalarExpression::new(
        query_location,
        inner_expression,
    )))
}

fn parse_replace_string_expression(
    replace_string_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&replace_string_expression_rule);

    let mut replace_string_rules = replace_string_expression_rule.into_inner();

    return Ok(ScalarExpression::Text(TextScalarExpression::Replace(
        ReplaceTextScalarExpression::new(
            query_location,
            parse_and_validate_string_rule(replace_string_rules.next().unwrap(), scope)?,
            parse_and_validate_string_rule(replace_string_rules.next().unwrap(), scope)?,
            parse_and_validate_string_rule(replace_string_rules.next().unwrap(), scope)?,
            false, // case_insensitive - set to false for KQL
        ),
    )));

    fn parse_and_validate_string_rule(
        rule: Pair<Rule>,
        scope: &dyn ParserScope,
    ) -> Result<ScalarExpression, ParserError> {
        let location = to_query_location(&rule);

        let mut scalar = parse_scalar_expression(rule, scope)?;

        if let Some(v) = scope.try_resolve_value_type(&mut scalar)?
            && v != ValueType::String
        {
            return Err(ParserError::QueryLanguageDiagnostic {
                location,
                diagnostic_id: "KS107",
                message: "A value of type string expected".into(),
            });
        }

        Ok(scalar)
    }
}

fn parse_substring_expression(
    substring_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&substring_expression_rule);

    let mut substring_rules = substring_expression_rule.into_inner();
    let source_scalar = parse_scalar_expression(substring_rules.next().unwrap(), scope)?;

    let starting_index_rule = substring_rules.next().unwrap();
    let starting_index_rule_location = to_query_location(&starting_index_rule);
    let mut starting_index_scalar = parse_scalar_expression(starting_index_rule, scope)?;

    if let Some(v) = scope.try_resolve_value_type(&mut starting_index_scalar)?
        && v != ValueType::Integer
    {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: starting_index_rule_location,
            diagnostic_id: "KS134",
            message: "The expression value must be an integer".into(),
        });
    }

    let length_scalar = if let Some(length_rule) = substring_rules.next() {
        let length_scalar_rule_location = to_query_location(&length_rule);
        let mut length_scalar = parse_scalar_expression(length_rule, scope)?;

        if let Some(v) = scope.try_resolve_value_type(&mut length_scalar)?
            && v != ValueType::Integer
        {
            return Err(ParserError::QueryLanguageDiagnostic {
                location: length_scalar_rule_location,
                diagnostic_id: "KS134",
                message: "The expression value must be an integer".into(),
            });
        }

        Some(length_scalar)
    } else {
        None
    };

    Ok(ScalarExpression::Slice(SliceScalarExpression::new(
        query_location,
        source_scalar,
        Some(starting_index_scalar),
        length_scalar,
    )))
}

fn parse_strcat_expression(
    strcat_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&strcat_expression_rule);

    let strcat_rules = strcat_expression_rule
        .into_inner()
        .next()
        .unwrap() // Note: We expect first rule to be scalar_list_expression
        .into_inner();

    let mut values = Vec::new();

    for rule in strcat_rules {
        let scalar = parse_scalar_expression(rule, scope)?;

        values.push(scalar);
    }

    Ok(ScalarExpression::Text(TextScalarExpression::Concat(
        CombineScalarExpression::new(
            query_location.clone(),
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(query_location, values),
            )),
        ),
    )))
}

fn parse_strcat_delim_expression(
    strcat_delim_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&strcat_delim_expression_rule);

    let mut strcat_delim_rules = strcat_delim_expression_rule.into_inner();

    let separator = parse_scalar_expression(strcat_delim_rules.next().unwrap(), scope)?;

    let mut values = Vec::new();

    for rule in strcat_delim_rules {
        let scalar = parse_scalar_expression(rule, scope)?;

        values.push(scalar);
    }

    Ok(ScalarExpression::Text(TextScalarExpression::Join(
        JoinTextScalarExpression::new(
            query_location.clone(),
            separator,
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(query_location, values),
            )),
        ),
    )))
}

fn parse_extract_expression(
    extract_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&extract_expression_rule);

    let mut extract_rules = extract_expression_rule.into_inner();

    let regex_rule = extract_rules.next().unwrap();
    let regex_location = to_query_location(&regex_rule);
    let mut regex = parse_scalar_expression(regex_rule, scope)?;
    if let Some(v) = scope.try_resolve_value_type(&mut regex)?
        && v != ValueType::Regex
        && v != ValueType::String
    {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: regex_location,
            diagnostic_id: "KS107",
            message: "A value of type regex or string expected".into(),
        });
    }

    let capture_rule = extract_rules.next().unwrap();
    let capture_location = to_query_location(&capture_rule);
    let mut capture_group = parse_scalar_expression(capture_rule, scope)?;

    if let Some(v) = scope.try_resolve_value_type(&mut capture_group)?
        && v != ValueType::Integer
        && v != ValueType::String
    {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: capture_location,
            diagnostic_id: "KS107",
            message: "A value of type long or string expected".into(),
        });
    }

    let source_rule = extract_rules.next().unwrap();
    let source_location = to_query_location(&source_rule);
    let mut source = parse_scalar_expression(source_rule, scope)?;

    if let Some(v) = scope.try_resolve_value_type(&mut source)?
        && v != ValueType::String
    {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: source_location,
            diagnostic_id: "KS107",
            message: "A value of type string expected".into(),
        });
    }

    Ok(ScalarExpression::Text(TextScalarExpression::Capture(
        CaptureTextScalarExpression::new(query_location, source, regex, capture_group),
    )))
}

#[cfg(test)]
mod tests {
    use pest::Parser;
    use regex::Regex;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_pest_parse_string_scalar_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::scalar_expression,
            &[
                "strlen(\"hello\")",
                "replace_string(\"text\", \"old\", \"new\")",
                "substring(\"hello world\", 0)",
                "substring(\"hello world\", 0, 5)",
                "parse_json(\"{\\\"key\\\": \\\"value\\\"}\")",
            ],
            &["!"],
        );
    }

    #[test]
    fn test_parse_strlen_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let error = parse_scalar_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::QueryLanguageDiagnostic {
                location: _,
                diagnostic_id: id,
                message: msg,
            } = error
            {
                assert_eq!(expected_id, id);
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected QueryLanguageDiagnostic");
            }
        };

        run_test_success(
            "strlen(\"hello\")",
            ScalarExpression::Length(LengthScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                )),
            )),
        );

        run_test_failure(
            "strlen(1)",
            "KS141",
            "The expression must have one of the types: string or dynamic",
        );
    }

    #[test]
    fn test_parse_replace_string_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let error = parse_scalar_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::QueryLanguageDiagnostic {
                location: _,
                diagnostic_id: id,
                message: msg,
            } = error
            {
                assert_eq!(expected_id, id);
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected QueryLanguageDiagnostic");
            }
        };

        run_test_success(
            "replace_string(\"A magic trick can turn a cat into a dog\", \"cat\", \"hamster\")",
            ScalarExpression::Text(TextScalarExpression::Replace(
                ReplaceTextScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "A magic trick can turn a cat into a dog",
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "cat"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hamster"),
                    )),
                    false, // case_insensitive
                ),
            )),
        );

        run_test_failure(
            "replace_string(1, 'hello', 'world')",
            "KS107",
            "A value of type string expected",
        );
        run_test_failure(
            "replace_string('hello world', 1, 'world')",
            "KS107",
            "A value of type string expected",
        );
        run_test_failure(
            "replace_string('hello world', 'hello', 1)",
            "KS107",
            "A value of type string expected",
        );
    }

    #[test]
    fn test_parse_substring_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let error = parse_scalar_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::QueryLanguageDiagnostic {
                location: _,
                diagnostic_id: id,
                message: msg,
            } = error
            {
                assert_eq!(expected_id, id);
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected QueryLanguageDiagnostic");
            }
        };

        run_test_success(
            "substring('asdf', 0)",
            ScalarExpression::Slice(SliceScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "asdf"),
                )),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                ))),
                None,
            )),
        );

        run_test_success(
            "substring('asdf', 0, 1)",
            ScalarExpression::Slice(SliceScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "asdf"),
                )),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            )),
        );

        run_test_failure(
            "substring('asdf', '0')",
            "KS134",
            "The expression value must be an integer",
        );

        run_test_failure(
            "substring('asdf', 0, '0')",
            "KS134",
            "The expression value must be an integer",
        );
    }

    #[test]
    fn test_parse_strcat_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "strcat(\"hello\")",
            ScalarExpression::Text(TextScalarExpression::Concat(CombineScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Collection(CollectionScalarExpression::List(
                    ListScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                        ))],
                    ),
                )),
            ))),
        );

        run_test_success(
            "strcat(\"hello\", 'world')",
            ScalarExpression::Text(TextScalarExpression::Concat(CombineScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Collection(CollectionScalarExpression::List(
                    ListScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "world"),
                            )),
                        ],
                    ),
                )),
            ))),
        );
    }

    #[test]
    fn test_parse_strcat_delim_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "strcat_delim(\"|\", \"hello\")",
            ScalarExpression::Text(TextScalarExpression::Join(JoinTextScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "|"),
                )),
                ScalarExpression::Collection(CollectionScalarExpression::List(
                    ListScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                        ))],
                    ),
                )),
            ))),
        );

        run_test_success(
            "strcat_delim(\", \", \"hello\", 'world')",
            ScalarExpression::Text(TextScalarExpression::Join(JoinTextScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ", "),
                )),
                ScalarExpression::Collection(CollectionScalarExpression::List(
                    ListScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "world"),
                            )),
                        ],
                    ),
                )),
            ))),
        );
    }

    #[test]
    fn test_parse_extract_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let mut expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            expression
                .try_resolve_static(&state.get_pipeline().get_resolution_scope())
                .unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: Option<&str>, expected_msg: &str| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let error = match parse_scalar_expression(result.next().unwrap(), &state) {
                Err(e) => e,
                Ok(mut s) => s
                    .try_resolve_static(&state.get_pipeline().get_resolution_scope())
                    .map_err(|e| ParserError::from(&e))
                    .unwrap_err(),
            };

            if let Some(eid) = expected_id {
                if let ParserError::QueryLanguageDiagnostic {
                    location: _,
                    diagnostic_id: id,
                    message: msg,
                } = error
                {
                    assert_eq!(eid, id);
                    assert_eq!(expected_msg, msg);
                } else {
                    panic!("Expected QueryLanguageDiagnostic");
                }
            } else if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Unexpected error");
            }
        };

        // Note: This gets folded into a string.
        run_test_success(
            r#"extract('(\\w*)', 1, 'hello world')"#,
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            ))),
        );

        run_test_success(
            r#"extract('(\\w*)', 1, SomeField)"#,
            ScalarExpression::Text(TextScalarExpression::Capture(
                CaptureTextScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "SomeField",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Regex(
                        RegexScalarExpression::new(
                            QueryLocation::new_fake(),
                            Regex::new("(\\w*)").unwrap(),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                ),
            )),
        );

        run_test_failure(
            r#"extract(timespan(1h), 1, 'hello world')"#,
            Some("KS107"),
            "A value of type regex or string expected",
        );

        run_test_failure(
            r#"extract('hello world', timespan(1h), 'hello world')"#,
            Some("KS107"),
            "A value of type long or string expected",
        );

        run_test_failure(
            r#"extract('hello world', 'name', timespan(1h))"#,
            Some("KS107"),
            "A value of type string expected",
        );

        run_test_failure(
            r#"extract('(', 0, 'hello world')"#,
            None,
            "Failed to parse Regex from pattern: regex parse error:\n    (\n    ^\nerror: unclosed group",
        );
    }
}
