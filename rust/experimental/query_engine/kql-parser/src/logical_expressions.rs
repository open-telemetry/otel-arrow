// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_logical_expression(
    logical_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<LogicalExpression, ParserError> {
    let scalar = parse_scalar_expression(logical_expression_rule, scope)?;

    to_logical_expression(scalar, scope)
}

pub(crate) fn to_logical_expression(
    mut scalar: ScalarExpression,
    scope: &dyn ParserScope,
) -> Result<LogicalExpression, ParserError> {
    if let ScalarExpression::Logical(l) = scalar {
        Ok(*l)
    } else {
        if let Some(v) = scope.try_resolve_value_type(&mut scalar)?
            && v != ValueType::Boolean
        {
            return Err(ParserError::QueryLanguageDiagnostic {
                location: scalar.get_query_location().clone(),
                diagnostic_id: "KS141",
                message: "The expression must have the type bool".into(),
            });
        }

        Ok(LogicalExpression::Scalar(scalar))
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;
    use regex::Regex;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_pest_parse_comparison_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::logical_expression,
            &[
                "1 == 1",
                "1 =~ 1",
                "(1) != true",
                "1 !~ true",
                "(1==1) > false",
                "1 >= 1",
                "1 < 1",
                "1 <= 1",
                "field contains 'value'",
                "field contains_cs 'value'",
                "field has 'value'",
                "field has_cs 'value'",
                "field !contains 'value'",
                "field !contains_cs 'value'",
                "field !has 'value'",
                "field !has_cs 'value'",
                "field in ('value1', 'value2')",
                "field in~ ('value1', 'value2')",
                "field !in ('value1', 'value2')",
                "field !in~ ('value1', 'value2')",
            ],
            &[],
        );
    }

    #[test]
    fn test_parse_comparison_expression() {
        let run_test = |input: &str, expected: LogicalExpression| {
            println!("Testing: {input}");

            let state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            state.push_constant(
                "const_regex",
                StaticScalarExpression::Regex(RegexScalarExpression::new(
                    QueryLocation::new_fake(),
                    Regex::new(".*").unwrap(),
                )),
            );

            let mut result = KqlPestParser::parse(Rule::logical_expression, input).unwrap();

            let mut expression = parse_logical_expression(result.next().unwrap(), &state).unwrap();

            expression
                .try_resolve_static(&state.get_pipeline().get_resolution_scope())
                .unwrap();

            assert_eq!(expected, expression);
        };

        run_test(
            "variable == 'hello world'",
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                false,
            )),
        );

        run_test(
            "variable =~ 'hello world'",
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                true,
            )),
        );

        run_test(
            "variable !~ 'hello world'",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    true,
                )),
            )),
        );

        run_test(
            "variable != true",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
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
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "source_path",
                        )),
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
                    StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key",
                        )),
                    )]),
                )),
            )),
        );

        run_test(
            "0 < (variable)",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::GreaterThanOrEqualTo(
                    GreaterThanOrEqualToLogicalExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )),
                        ScalarExpression::Variable(VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                            ValueAccessor::new(),
                        )),
                    ),
                ),
            )),
        );

        run_test(
            "(0) <= variable",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    )),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
                    )),
                )),
            )),
        );

        // Note: This whole expression folds to a constant value.
        run_test(
            "'hello world' matches regex '^hello world$'",
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
        );

        // Note: The string regex pattern gets folded into a compiled regex
        // expression.
        run_test(
            "Name matches regex '^hello world$'",
            LogicalExpression::Matches(MatchesLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "Name",
                        )),
                    )]),
                )),
                ScalarExpression::Static(StaticScalarExpression::Regex(
                    RegexScalarExpression::new(
                        QueryLocation::new_fake(),
                        Regex::new("^hello world$").unwrap(),
                    ),
                )),
            )),
        );

        run_test(
            "Name matches regex const_regex",
            LogicalExpression::Matches(MatchesLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "Name",
                        )),
                    )]),
                )),
                ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueType::Regex,
                    0,
                    ValueAccessor::new(),
                )),
            )),
        );
    }

    #[test]
    fn test_pest_parse_logical_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
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
            println!("Testing: {input}");

            let state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlPestParser::parse(Rule::logical_expression, input).unwrap();

            let expression = parse_logical_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::logical_expression, input).unwrap();

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
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "name",
                    )),
                )]),
            ))),
        );

        run_test_success(
            "resource.attributes['service.name']",
            LogicalExpression::Scalar(ScalarExpression::Attached(AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "service.name"),
                    )),
                ]),
            ))),
        );

        run_test_success(
            "variable",
            LogicalExpression::Scalar(ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                ValueAccessor::new(),
            ))),
        );

        run_test_success(
            "variable == 'hello world'",
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                false,
            )),
        );

        run_test_success(
            "variable == 'hello world' or SeverityText != 'Info'",
            LogicalExpression::Or(OrLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    false,
                )),
                LogicalExpression::Not(NotLogicalExpression::new(
                    QueryLocation::new_fake(),
                    LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "SeverityText",
                                )),
                            )]),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "Info"),
                        )),
                        false,
                    )),
                )),
            )),
        );

        run_test_success(
            "(variable == ('hello world') or (SeverityNumber) >= 0) and (true)",
            LogicalExpression::And(AndLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Or(OrLogicalExpression::new(
                    QueryLocation::new_fake(),
                    LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Variable(VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                            ValueAccessor::new(),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                        )),
                        false,
                    )),
                    LogicalExpression::GreaterThanOrEqualTo(
                        GreaterThanOrEqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                    StaticScalarExpression::String(StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "SeverityNumber",
                                    )),
                                )]),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                            )),
                        ),
                    ),
                )),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
            )),
        );

        run_test_failure(
            "'hello world'",
            "KS141",
            "The expression must have the type bool",
        );
    }

    #[test]
    fn test_parse_contains_expressions() {
        let run_test = |input: &str, expected: LogicalExpression| {
            let state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");
            state.push_variable_name("var1");

            let mut result = KqlPestParser::parse(Rule::logical_expression, input).unwrap();

            let expression = parse_logical_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        // Test contains
        run_test(
            "variable contains 'test'",
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "test"),
                )),
                true, // case_insensitive
            )),
        );

        // Test contains_cs
        run_test(
            "variable contains_cs 'test'",
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "test"),
                )),
                false, // case_sensitive
            )),
        );

        // Test has
        run_test(
            "variable has 'test'",
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "test"),
                )),
                true, // case_insensitive
            )),
        );

        // Test !contains
        run_test(
            "variable !contains 'test'",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Contains(ContainsLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "test"),
                    )),
                    true,
                )),
            )),
        );

        // Test in operator
        run_test(
            "variable in (var1, 'test2')",
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Collection(CollectionScalarExpression::List(
                    ListScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![
                            ScalarExpression::Variable(VariableScalarExpression::new(
                                QueryLocation::new_fake(),
                                StringScalarExpression::new(QueryLocation::new_fake(), "var1"),
                                ValueAccessor::new(),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "test2"),
                            )),
                        ],
                    ),
                )),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                false, // in is case_sensitive by default
            )),
        );

        // Test in~ operator (case insensitive)
        run_test(
            "variable in~ ('test1', 'test2')",
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Collection(CollectionScalarExpression::List(
                    ListScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "test1"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "test2"),
                            )),
                        ],
                    ),
                )),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                true, // in~ is case_insensitive
            )),
        );

        // Test !in operator
        run_test(
            "variable !in ('test1', 'test2')",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Contains(ContainsLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Collection(CollectionScalarExpression::List(
                        ListScalarExpression::new(
                            QueryLocation::new_fake(),
                            vec![
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "test1"),
                                )),
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "test2"),
                                )),
                            ],
                        ),
                    )),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
                    )),
                    false,
                )),
            )),
        );

        // Test !in~ operator (case insensitive)
        run_test(
            "variable !in~ ('test1', 'test2')",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Contains(ContainsLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Collection(CollectionScalarExpression::List(
                        ListScalarExpression::new(
                            QueryLocation::new_fake(),
                            vec![
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "test1"),
                                )),
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "test2"),
                                )),
                            ],
                        ),
                    )),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
                    )),
                    true, // !in~ is case_insensitive
                )),
            )),
        );
    }
}
