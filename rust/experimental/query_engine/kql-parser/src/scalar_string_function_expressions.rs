use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_strlen_expression(
    strlen_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&strlen_expression_rule);

    let mut strlen_rules = strlen_expression_rule.into_inner();
    let inner_expression = strlen_rules.next().unwrap();

    Ok(ScalarExpression::Length(LengthScalarExpression::new(
        query_location,
        parse_scalar_expression(inner_expression, state)?,
    )))
}

pub(crate) fn parse_replace_string_expression(
    replace_string_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&replace_string_expression_rule);

    let mut replace_string_rules = replace_string_expression_rule.into_inner();
    let haystack_expression = replace_string_rules.next().unwrap();
    let needle_expression = replace_string_rules.next().unwrap();
    let replacement_expression = replace_string_rules.next().unwrap();

    Ok(ScalarExpression::Text(TextScalarExpression::Replace(
        ReplaceTextScalarExpression::new(
            query_location,
            parse_scalar_expression(haystack_expression, state)?,
            parse_scalar_expression(needle_expression, state)?,
            parse_scalar_expression(replacement_expression, state)?,
            false, // case_insensitive - set to false for KQL
        ),
    )))
}

pub(crate) fn parse_substring_expression(
    substring_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&substring_expression_rule);

    let mut substring_rules = substring_expression_rule.into_inner();
    let source_scalar = parse_scalar_expression(substring_rules.next().unwrap(), state)?;
    let starting_index_scalar = parse_scalar_expression(substring_rules.next().unwrap(), state)?;
    let length_scalar = substring_rules
        .next()
        .map(|v| parse_scalar_expression(v, state))
        .transpose()?;

    let starting_index_value_type_result = starting_index_scalar
        .try_resolve_value_type(state.get_pipeline())
        .map_err(|e| ParserError::from(&e))?;

    if let Some(v) = starting_index_value_type_result
        && v != ValueType::Integer
    {
        return Err(ParserError::QueryLanguageDiagnostic {
            location: query_location,
            diagnostic_id: "KS134",
            message: "The expression value must be an integer".into(),
        });
    }

    if let Some(s) = length_scalar.as_ref() {
        let length_scalar_value_type_result = s
            .try_resolve_value_type(state.get_pipeline())
            .map_err(|e| ParserError::from(&e))?;
        if let Some(v) = length_scalar_value_type_result
            && v != ValueType::Integer
        {
            return Err(ParserError::QueryLanguageDiagnostic {
                location: query_location,
                diagnostic_id: "KS134",
                message: "The expression value must be an integer".into(),
            });
        }
    }

    Ok(ScalarExpression::Slice(SliceScalarExpression::new(
        query_location,
        source_scalar,
        Some(starting_index_scalar),
        length_scalar,
    )))
}

#[cfg(test)]
mod tests {
    use pest::Parser;

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

        run_test_success(
            "strlen(\"hello\")",
            ScalarExpression::Length(LengthScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                )),
            )),
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
}
