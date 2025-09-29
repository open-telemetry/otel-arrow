// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_parse_unary_expressions(
    parse_unary_expressions_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let rule = parse_unary_expressions_rule.into_inner().next().unwrap();

    match rule.as_rule() {
        Rule::parse_json_expression => parse_parse_json_expression(rule, scope),
        Rule::parse_regex_expression => parse_parse_regex_expression(rule, scope),
        _ => panic!("Unexpected rule in parse_unary_expressions: {rule}"),
    }
}

fn parse_parse_json_expression(
    parse_json_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&parse_json_expression_rule);

    let mut parse_json_rules = parse_json_expression_rule.into_inner();

    let inner_rule = parse_json_rules.next().unwrap();
    let inner_rule_location = to_query_location(&inner_rule);
    let mut inner_scalar = parse_scalar_expression(inner_rule, scope)?;

    if let Some(v) = scope.try_resolve_value_type(&mut inner_scalar)?
        && v != ValueType::String
    {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: inner_rule_location,
            diagnostic_id: "KS107",
            message: "A value of type string expected".into(),
        });
    }

    let mut parse_json_scalar = ScalarExpression::Parse(ParseScalarExpression::Json(
        ParseJsonScalarExpression::new(query_location, inner_scalar),
    ));

    // Note: Call into try_resolve_static here is to try to validate the json
    // string and bubble up any errors. It can be removed if/when expression
    // tree gets constant folding which should automatically call into
    // try_resolve_static.
    parse_json_scalar
        .try_resolve_static(&scope.get_pipeline().get_resolution_scope())
        .map_err(|e| ParserError::from(&e))?;

    Ok(parse_json_scalar)
}

fn parse_parse_regex_expression(
    parse_regex_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&parse_regex_expression_rule);

    let mut inner_rules = parse_regex_expression_rule.into_inner();

    let pattern_rule = inner_rules.next().unwrap();
    let pattern_location = to_query_location(&pattern_rule);
    let mut pattern = parse_scalar_expression(pattern_rule, scope)?;

    if let Some(v) = scope.try_resolve_value_type(&mut pattern)?
        && v != ValueType::String
    {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: pattern_location,
            diagnostic_id: "KS107",
            message: "A value of type string expected".into(),
        });
    }

    let options = if let Some(options_rule) = inner_rules.next() {
        let options_location = to_query_location(&options_rule);
        let mut options = parse_scalar_expression(options_rule, scope)?;

        if let Some(v) = scope.try_resolve_value_type(&mut options)?
            && v != ValueType::String
        {
            return Err(ParserError::QueryLanguageDiagnostic {
                location: options_location,
                diagnostic_id: "KS107",
                message: "A value of type string expected".into(),
            });
        }

        Some(options)
    } else {
        None
    };

    Ok(ScalarExpression::Parse(ParseScalarExpression::Regex(
        ParseRegexScalarExpression::new(query_location, pattern, options),
    )))
}

#[cfg(test)]
mod tests {
    use pest::Parser;
    use regex::Regex;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_parse_parse_json_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: Option<&str>, expected_msg: &str| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let error = parse_scalar_expression(result.next().unwrap(), &state).unwrap_err();

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

        // Note: This whole expression is folded into a static array.
        run_test_success(
            "parse_json('[]')",
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            ))),
        );

        run_test_success(
            "parse_json(source.Attributes['json'])",
            ScalarExpression::Parse(ParseScalarExpression::Json(ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "Attributes"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "json"),
                        )),
                    ]),
                )),
            ))),
        );

        run_test_failure(
            "parse_json(0)",
            Some("KS107"),
            "A value of type string expected",
        );

        run_test_failure(
            "parse_json('[')",
            None,
            "Input could not be parsed as JSON: EOF while parsing a list at line 1 column 1",
        );
    }

    #[test]
    fn test_parse_parse_regex_expression() {
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

        // Note: This whole expression is folded into a static regex.
        run_test_success(
            "parse_regex('hello')",
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new("hello").unwrap(),
            ))),
        );

        // Note: This whole expression is folded into a static regex.
        run_test_success(
            "parse_regex('hello', 'i')",
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new("hello").unwrap(),
            ))),
        );

        run_test_success(
            "parse_regex(SomeRegexField)",
            ScalarExpression::Parse(ParseScalarExpression::Regex(
                ParseRegexScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "SomeRegexField",
                            )),
                        )]),
                    )),
                    None,
                ),
            )),
        );

        run_test_success(
            "parse_regex(SomeRegexField, SomeRegexOptionsField)",
            ScalarExpression::Parse(ParseScalarExpression::Regex(
                ParseRegexScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "SomeRegexField",
                            )),
                        )]),
                    )),
                    Some(ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "SomeRegexOptionsField",
                            )),
                        )]),
                    ))),
                ),
            )),
        );

        run_test_failure(
            "parse_regex(1)",
            Some("KS107"),
            "A value of type string expected",
        );

        run_test_failure(
            "parse_regex('a', 1)",
            Some("KS107"),
            "A value of type string expected",
        );

        run_test_failure(
            "parse_regex('(')",
            None,
            "Failed to parse Regex from pattern: regex parse error:\n    (\n    ^\nerror: unclosed group",
        );
    }
}
