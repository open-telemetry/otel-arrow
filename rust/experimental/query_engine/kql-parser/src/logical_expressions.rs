use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_comparison_expression(
    comparison_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&comparison_expression_rule);

    let mut comparison_rules = comparison_expression_rule.into_inner();

    let left_rule = comparison_rules.next().unwrap();

    let left = match left_rule.as_rule() {
        Rule::scalar_expression => parse_scalar_expression(left_rule, state)?,
        _ => panic!("Unexpected rule in comparison_expression: {left_rule}"),
    };

    let operation_rule = comparison_rules.next().unwrap();

    let right_rule = comparison_rules.next().unwrap();

    let right = match right_rule.as_rule() {
        Rule::scalar_expression => parse_scalar_expression(right_rule, state)?,
        _ => panic!("Unexpected rule in comparison_expression: {right_rule}"),
    };

    match operation_rule.as_rule() {
        Rule::equals_token => Ok(LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            query_location,
            left,
            right,
        ))),
        Rule::not_equals_token => Ok(LogicalExpression::Not(NotLogicalExpression::new(
            query_location.clone(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(query_location, left, right)),
        ))),

        Rule::greater_than_token => Ok(LogicalExpression::GreaterThan(
            GreaterThanLogicalExpression::new(query_location, left, right),
        )),
        Rule::greater_than_or_equal_to_token => Ok(LogicalExpression::GreaterThanOrEqualTo(
            GreaterThanOrEqualToLogicalExpression::new(query_location, left, right),
        )),

        Rule::less_than_token => {
            Ok(LogicalExpression::Not(NotLogicalExpression::new(
                query_location.clone(),
                LogicalExpression::GreaterThanOrEqualTo(
                    GreaterThanOrEqualToLogicalExpression::new(query_location, left, right),
                ),
            )))
        }
        Rule::less_than_or_equal_to_token => Ok(LogicalExpression::Not(NotLogicalExpression::new(
            query_location.clone(),
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                query_location,
                left,
                right,
            )),
        ))),

        _ => panic!("Unexpected rule in operation_rule: {operation_rule}"),
    }
}

pub(crate) fn parse_logical_expression(
    logical_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&logical_expression_rule);

    let mut logical_rules = logical_expression_rule.into_inner();

    let parse_rule =
        |logical_expression_rule: Pair<Rule>| -> Result<LogicalExpression, ParserError> {
            match logical_expression_rule.as_rule() {
                Rule::comparison_expression => {
                    Ok(parse_comparison_expression(logical_expression_rule, state)?)
                }
                Rule::scalar_expression => {
                    let scalar = parse_scalar_expression(logical_expression_rule, state)?;

                    if let ScalarExpression::Logical(l) = scalar {
                        Ok(*l)
                    } else {
                        let value = scalar.to_value();
                        if value.is_some() && !matches!(value, Some(Value::Boolean(_))) {
                            return Err(ParserError::QueryLanguageDiagnostic {
                                location: scalar.get_query_location().clone(),
                                diagnostic_id: "KS141",
                                message: "The expression must have the type bool".into(),
                            });
                        }

                        Ok(LogicalExpression::Scalar(scalar))
                    }
                }
                _ => {
                    panic!("Unexpected rule in logical_expression_rule: {logical_expression_rule}")
                }
            }
        };

    let first_expression = parse_rule(logical_rules.next().unwrap())?;

    let mut chain_rules = Vec::new();
    loop {
        let rule = logical_rules.next();
        if rule.is_none() {
            break;
        }

        let chain_rule = rule.unwrap();
        match chain_rule.as_rule() {
            Rule::and_token => chain_rules.push(ChainedLogicalExpression::And(parse_rule(
                logical_rules.next().unwrap(),
            )?)),
            Rule::or_token => chain_rules.push(ChainedLogicalExpression::Or(parse_rule(
                logical_rules.next().unwrap(),
            )?)),
            _ => panic!("Unexpected rule in chain_rule: {chain_rule}"),
        }
    }

    if !chain_rules.is_empty() {
        let mut chain = ChainLogicalExpression::new(query_location, first_expression);

        for expression in chain_rules {
            match expression {
                ChainedLogicalExpression::Or(logical_expression) => {
                    chain.push_or(logical_expression)
                }
                ChainedLogicalExpression::And(logical_expression) => {
                    chain.push_and(logical_expression)
                }
            };
        }

        return Ok(LogicalExpression::Chain(chain));
    }

    Ok(first_expression)
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlParser;

    use super::*;

    #[test]
    fn test_pest_parse_comparison_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::comparison_expression,
            &[
                "1 == 1",
                "(1) != true",
                "(1==1) > false",
                "1 >= 1",
                "1 < 1",
                "1 <= 1",
            ],
            &[],
        );
    }

    #[test]
    fn test_parse_comparison_expression() {
        let run_test = |input: &str, expected: LogicalExpression| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::comparison_expression, input).unwrap();

            let expression = parse_comparison_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test(
            "variable == 'hello world'",
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            )),
        );

        run_test(
            "(true == true) != true",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Logical(
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Boolean(
                                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Boolean(
                                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                            )),
                        ))
                        .into(),
                    ),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                )),
            )),
        );

        run_test(
            "1 > source_path",
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                        StringScalarExpression::new(QueryLocation::new_fake(), "source_path"),
                    )]),
                )),
            )),
        );

        run_test(
            "(1) >= resource.key",
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Attached(AttachedScalarExpression::new(
                    QueryLocation::new_fake(),
                    "resource",
                    ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key"),
                    )]),
                )),
            )),
        );

        run_test(
            "0 < (1)",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::GreaterThanOrEqualTo(
                    GreaterThanOrEqualToLogicalExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                        )),
                    ),
                ),
            )),
        );

        run_test(
            "0 <= (true == true)",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    )),
                    ScalarExpression::Logical(
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Boolean(
                                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Boolean(
                                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                            )),
                        ))
                        .into(),
                    ),
                )),
            )),
        );
    }

    #[test]
    fn test_pest_parse_logical_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::logical_expression,
            &[
                "true",
                "false",
                "variable",
                "a == b",
                "a == b or 10 < 1",
                "(true)",
                "(true or variable['a'])",
                "(variable['a'] == 'hello' or variable.b == 'world') and datetime(6/1/2025) > datetime(1/1/2025)",
            ],
            &["!"],
        );
    }

    #[test]
    fn test_parse_logical_expression() {
        let run_test_success = |input: &str, expected: LogicalExpression| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::logical_expression, input).unwrap();

            let expression = parse_logical_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            let state = ParserState::new(input);

            let mut result = KqlParser::parse(Rule::logical_expression, input).unwrap();

            let error = parse_logical_expression(result.next().unwrap(), &state).unwrap_err();

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
            "true",
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
        );

        run_test_success(
            "false",
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            ))),
        );

        run_test_success(
            "source.name",
            LogicalExpression::Scalar(ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                    StringScalarExpression::new(QueryLocation::new_fake(), "name"),
                )]),
            ))),
        );

        run_test_success(
            "resource.attributes['service.name']",
            LogicalExpression::Scalar(ScalarExpression::Attached(AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                "resource",
                ValueAccessor::new_with_selectors(vec![
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "attributes",
                    )),
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "service.name",
                    )),
                ]),
            ))),
        );

        run_test_success(
            "variable",
            LogicalExpression::Scalar(ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                "variable",
                ValueAccessor::new(),
            ))),
        );

        run_test_success(
            "variable == 'hello world'",
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            )),
        );

        let mut chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            )),
        );

        chain.push_or(LogicalExpression::Not(NotLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                        StringScalarExpression::new(QueryLocation::new_fake(), "SeverityText"),
                    )]),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "Info"),
                )),
            )),
        )));

        run_test_success(
            "variable == 'hello world' or SeverityText != 'Info'",
            LogicalExpression::Chain(chain),
        );

        let mut nested_logical = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            )),
        );

        nested_logical.push_or(LogicalExpression::GreaterThanOrEqualTo(
            GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                        StringScalarExpression::new(QueryLocation::new_fake(), "SeverityNumber"),
                    )]),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
        ));

        let mut nested_chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Chain(nested_logical),
        );

        nested_chain.push_and(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
        )));

        run_test_success(
            "(variable == ('hello world') or (SeverityNumber) >= 0) and (true)",
            LogicalExpression::Chain(nested_chain),
        );

        run_test_failure(
            "'hello world'",
            "KS141",
            "The expression must have the type bool",
        );
    }
}
