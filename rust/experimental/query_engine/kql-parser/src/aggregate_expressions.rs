// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_aggregate_assignment_expression(
    aggregate_assignment_expression_rule: Pair<Rule>,
    state: &dyn ParserScope,
) -> Result<(Box<str>, AggregationExpression), ParserError> {
    let mut aggregate_assignment_rules = aggregate_assignment_expression_rule.into_inner();

    let destination_rule = aggregate_assignment_rules.next().unwrap();
    let destination_rule_str = destination_rule.as_str();

    let aggregation_expression =
        parse_aggregate_expression(aggregate_assignment_rules.next().unwrap(), state)?;

    Ok((destination_rule_str.into(), aggregation_expression))
}

fn parse_aggregate_expression(
    aggregate_expression_rule: Pair<Rule>,
    state: &dyn ParserScope,
) -> Result<AggregationExpression, ParserError> {
    let query_location = to_query_location(&aggregate_expression_rule);

    let aggregate = match aggregate_expression_rule.as_rule() {
        Rule::average_aggregate_expression => AggregationExpression::new(
            query_location,
            AggregationFunction::Average,
            Some(parse_scalar_expression(aggregate_expression_rule, state)?),
        ),
        Rule::count_aggregate_expression => {
            AggregationExpression::new(query_location, AggregationFunction::Count, None)
        }
        Rule::maximum_aggregate_expression => AggregationExpression::new(
            query_location,
            AggregationFunction::Maximum,
            Some(parse_scalar_expression(aggregate_expression_rule, state)?),
        ),
        Rule::minimum_aggregate_expression => AggregationExpression::new(
            query_location,
            AggregationFunction::Minimum,
            Some(parse_scalar_expression(aggregate_expression_rule, state)?),
        ),
        Rule::sum_aggregate_expression => AggregationExpression::new(
            query_location,
            AggregationFunction::Sum,
            Some(parse_scalar_expression(aggregate_expression_rule, state)?),
        ),
        _ => panic!("Unexpected rule in aggregate_expression: {aggregate_expression_rule}"),
    };

    Ok(aggregate)
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_parse_aggregate_assignment_expression() {
        let run_test_success = |input: &str, expected: (Box<str>, AggregationExpression)| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result =
                KqlPestParser::parse(Rule::aggregate_assignment_expression, input).unwrap();

            let expression =
                parse_aggregate_assignment_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "Count = count()",
            (
                "Count".into(),
                AggregationExpression::new(
                    QueryLocation::new_fake(),
                    AggregationFunction::Count,
                    None,
                ),
            ),
        );
    }

    #[test]
    fn test_parse_aggregate_expression() {
        let run_test_success = |input: &str, expected: AggregationExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::aggregate_expression, input).unwrap();

            let expression = parse_aggregate_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "avg(1)",
            AggregationExpression::new(
                QueryLocation::new_fake(),
                AggregationFunction::Average,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
        );

        run_test_success(
            "count()",
            AggregationExpression::new(QueryLocation::new_fake(), AggregationFunction::Count, None),
        );

        run_test_success(
            "max(1)",
            AggregationExpression::new(
                QueryLocation::new_fake(),
                AggregationFunction::Maximum,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
        );

        run_test_success(
            "min(1)",
            AggregationExpression::new(
                QueryLocation::new_fake(),
                AggregationFunction::Minimum,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
        );

        run_test_success(
            "sum(1)",
            AggregationExpression::new(
                QueryLocation::new_fake(),
                AggregationFunction::Sum,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
        );
    }
}
