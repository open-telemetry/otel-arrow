use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule, logical_expressions::parse_logical_expression, scalar_expression::parse_scalar_expression,
};

pub(crate) fn parse_conditional_expression(
    conditional_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&conditional_expression_rule);

    let mut conditional_rules = conditional_expression_rule.into_inner();

    let condition_logical = conditional_rules.next().unwrap();

    let true_scalar = conditional_rules.next().unwrap();

    let false_scalar = conditional_rules.next().unwrap();

    Ok(ScalarExpression::Conditional(
        ConditionalScalarExpression::new(
            query_location,
            parse_logical_expression(condition_logical, state)?,
            parse_scalar_expression(true_scalar, state)?,
            parse_scalar_expression(false_scalar, state)?,
        ),
    ))
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlParser;

    use super::*;

    #[test]
    fn test_parse_conditional_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            let state = ParserState::new(input);

            let mut result = KqlParser::parse(Rule::conditional_expression, input).unwrap();

            let expression = parse_conditional_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "iff(true, 1, 0)",
            ScalarExpression::Conditional(ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            )),
        );

        run_test_success(
            "iif(1 > 0, 1, 0)",
            ScalarExpression::Conditional(ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            )),
        );
    }
}
