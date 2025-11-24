// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, logical_expressions::parse_logical_expression};

pub(crate) fn parse_logical_unary_expressions(
    logical_unary_expressions_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let rule = logical_unary_expressions_rule.into_inner().next().unwrap();

    match rule.as_rule() {
        Rule::not_expression => parse_not_expression(rule, scope),
        _ => panic!("Unexpected rule in logical_unary_expressions: {rule}"),
    }
}

fn parse_not_expression(
    not_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&not_expression_rule);

    let mut not_rules = not_expression_rule.into_inner();
    let logical_expr_rule = not_rules.next().unwrap();

    let inner_logical = parse_logical_expression(logical_expr_rule, scope)?;

    Ok(ScalarExpression::Logical(
        LogicalExpression::Not(NotLogicalExpression::new(query_location, inner_logical)).into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KqlPestParser;
    use pest::Parser;

    #[test]
    fn test_pest_parse_not_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::not_expression,
            &[
                "not(true)",
                "not(false)",
                "not(1 == 1)",
                "not(field contains 'value')",
                "not(x > 5 and y < 10)",
            ],
            &[
                "not()",     // missing argument
                "not(1, 2)", // too many arguments
                "not true",  // missing parentheses
            ],
        );
    }

    #[test]
    fn test_parse_not_expression() {
        let run_test = |input: &str, expected: LogicalExpression| {
            println!("Testing: {input}");

            let state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            let mut result = KqlPestParser::parse(Rule::not_expression, input).unwrap();

            let result = parse_not_expression(result.next().unwrap(), &state).unwrap();

            let expected_scalar = ScalarExpression::Logical(expected.into());

            assert_eq!(result, expected_scalar);
        };

        run_test(
            "not(true)",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
            )),
        );

        run_test(
            "not(1 == 2)",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                    )),
                    false,
                )),
            )),
        );
    }
}
