// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule,
    scalar_expression::{parse_scalar_expression, try_resolve_identifier},
};

pub(crate) fn parse_aggregate_expression(
    aggregate_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<(Box<str>, AggregationExpression), ParserError> {
    let mut aggregate_expression_rules = aggregate_expression_rule.into_inner();

    let first_rule = aggregate_expression_rules.next().unwrap();
    let first_rule_location = to_query_location(&first_rule);

    match first_rule.as_rule() {
        Rule::identifier_literal => {
            let identifier = crate::tabular_expressions::validate_summary_identifier(
                scope,
                first_rule_location,
                &[first_rule.as_str().trim().into()],
            )?;

            let aggregation_expression =
                parse_aggregate_expressions(aggregate_expression_rules.next().unwrap(), scope)?;

            Ok((identifier.into(), aggregation_expression))
        }
        _ => {
            let aggregation_expression = parse_aggregate_expressions(first_rule, scope)?;

            let identifier = resolve_identifier(&aggregation_expression, scope)?;

            let full_identifier = crate::tabular_expressions::validate_summary_identifier(
                scope,
                first_rule_location,
                &identifier,
            )?;

            Ok((full_identifier.into(), aggregation_expression))
        }
    }
}

fn parse_aggregate_expressions(
    aggregate_expressions_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<AggregationExpression, ParserError> {
    let query_location = to_query_location(&aggregate_expressions_rule);

    let aggregate = match aggregate_expressions_rule.as_rule() {
        Rule::average_aggregate_expression => AggregationExpression::new(
            query_location,
            AggregationFunction::Average,
            Some(parse_scalar_expression(aggregate_expressions_rule, scope)?),
        ),
        Rule::count_aggregate_expression => {
            AggregationExpression::new(query_location, AggregationFunction::Count, None)
        }
        Rule::maximum_aggregate_expression => AggregationExpression::new(
            query_location,
            AggregationFunction::Maximum,
            Some(parse_scalar_expression(aggregate_expressions_rule, scope)?),
        ),
        Rule::minimum_aggregate_expression => AggregationExpression::new(
            query_location,
            AggregationFunction::Minimum,
            Some(parse_scalar_expression(aggregate_expressions_rule, scope)?),
        ),
        Rule::sum_aggregate_expression => AggregationExpression::new(
            query_location,
            AggregationFunction::Sum,
            Some(parse_scalar_expression(aggregate_expressions_rule, scope)?),
        ),
        _ => panic!("Unexpected rule in aggregate_expression: {aggregate_expressions_rule}"),
    };

    Ok(aggregate)
}

fn resolve_identifier(
    aggregation_expression: &AggregationExpression,
    scope: &dyn ParserScope,
) -> Result<Vec<Box<str>>, ParserError> {
    let f = match aggregation_expression.get_aggregation_function() {
        AggregationFunction::Average => "avg",
        AggregationFunction::Count => "count",
        AggregationFunction::Maximum => "max",
        AggregationFunction::Minimum => "min",
        AggregationFunction::Sum => "sum",
    };

    if let Some(s) = &aggregation_expression.get_value_expression()
        && let Some(mut i) = try_resolve_identifier(s, scope)?
    {
        i.insert(0, f.into());
        return Ok(i);
    }

    Ok(vec![f.into()])
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

            let mut result = KqlPestParser::parse(Rule::aggregate_expression, input).unwrap();

            let expression = parse_aggregate_expression(result.next().unwrap(), &state).unwrap();

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

            assert_eq!(expected, expression.1);
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
