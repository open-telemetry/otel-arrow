// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_now_expression(
    now_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&now_expression_rule);

    let mut now_rules = now_expression_rule.into_inner();

    match now_rules.next() {
        None => Ok(ScalarExpression::Temporal(TemporalScalarExpression::Now(
            NowScalarExpression::new(query_location),
        ))),
        Some(r) => match r.as_rule() {
            Rule::scalar_expression => {
                let offset = parse_scalar_expression(r, state)?;

                Ok(ScalarExpression::Math(MathScalarExpression::Add(
                    BinaryMathematicalScalarExpression::new(
                        query_location.clone(),
                        ScalarExpression::Temporal(TemporalScalarExpression::Now(
                            NowScalarExpression::new(query_location),
                        )),
                        offset,
                    ),
                )))
            }
            _ => panic!("Unexpected rule in now_expression: {r}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use pest::Parser;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_parse_now_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "now()",
            ScalarExpression::Temporal(TemporalScalarExpression::Now(NowScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "now(1h)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Temporal(TemporalScalarExpression::Now(
                        NowScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::hours(1),
                        ),
                    )),
                ),
            )),
        );
    }
}
