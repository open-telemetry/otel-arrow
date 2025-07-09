use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::Rule;

#[allow(dead_code)]
pub(crate) fn parse_scalar_expression(
    scalar_expression_rule: Pair<Rule>,
    _state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let scalar_rule = scalar_expression_rule.into_inner().next().unwrap();

    let scalar = match scalar_rule.as_rule() {
        Rule::true_literal | Rule::false_literal => {
            ScalarExpression::Static(parse_standard_bool_literal(scalar_rule))
        }
        Rule::float_literal => ScalarExpression::Static(parse_standard_float_literal(scalar_rule)?),
        Rule::integer_literal => {
            ScalarExpression::Static(parse_standard_integer_literal(scalar_rule)?)
        }
        Rule::string_literal => {
            ScalarExpression::Static(parse_standard_string_literal(scalar_rule))
        }
        Rule::scalar_expression => parse_scalar_expression(scalar_rule, _state)?,
        _ => panic!("Unexpected rule in scalar_expression: {scalar_rule}"),
    };

    Ok(scalar)
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::OttlPestParser;

    use super::*;

    #[test]
    fn test_parse_scalar_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            let state = ParserState::new(input);

            let mut result = OttlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "1",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        run_test_success(
            "(1)",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        run_test_success(
            "true",
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
        );

        run_test_success(
            "false",
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            )),
        );

        run_test_success(
            "\"hello world\"",
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
        );
    }
}
