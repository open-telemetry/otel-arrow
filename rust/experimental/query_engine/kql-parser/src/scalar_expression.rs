use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule, logical_expressions::parse_logical_expression,
    scalar_conditional_function_expressions::*, scalar_primitive_expressions::*,
};

pub(crate) fn parse_scalar_expression(
    scalar_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let scalar_rule = scalar_expression_rule.into_inner().next().unwrap();

    let scalar = match scalar_rule.as_rule() {
        Rule::real_expression => ScalarExpression::Static(parse_real_expression(scalar_rule)?),
        Rule::datetime_expression => {
            ScalarExpression::Static(parse_datetime_expression(scalar_rule)?)
        }
        Rule::true_literal | Rule::false_literal => {
            ScalarExpression::Static(parse_standard_bool_literal(scalar_rule))
        }
        Rule::double_literal => ScalarExpression::Static(parse_double_literal(scalar_rule)?),
        Rule::integer_literal => ScalarExpression::Static(parse_integer_literal(scalar_rule)?),
        Rule::string_literal => ScalarExpression::Static(parse_string_literal(scalar_rule)),
        Rule::accessor_expression => parse_accessor_expression(scalar_rule, state)?,
        Rule::logical_expression => {
            let l = parse_logical_expression(scalar_rule, state)?;

            if let LogicalExpression::Scalar(s) = l {
                s
            } else {
                ScalarExpression::Logical(l.into())
            }
        }
        Rule::conditional_expression => parse_conditional_expression(scalar_rule, state)?,
        Rule::scalar_expression => parse_scalar_expression(scalar_rule, state)?,
        _ => panic!("Unexpected rule in scalar_expression: {}", scalar_rule),
    };

    if matches!(&scalar, ScalarExpression::Static(_)) {
        return Ok(scalar);
    }

    let static_result = scalar.try_resolve_static();
    if let Err(e) = static_result {
        Err(ParserError::ExpressionError(e))
    } else if let Some(s) = static_result.unwrap() {
        Ok(ScalarExpression::Static(s))
    } else {
        Ok(scalar)
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::{KqlParser, date_utils::create_utc};

    use super::*;

    #[test]
    fn test_pest_parse_scalar_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::scalar_expression,
            &[
                "1",
                "1e1",
                "real(1)",
                "datetime(6/9/2025)",
                "true",
                "false",
                "(true == true)",
                "\"hello world\"",
                "variable",
                "(1)",
                "iff(true, 0, 1)",
            ],
            &["!"],
        );
    }

    #[test]
    fn test_parse_scalar_expression() {
        let run_test = |input: &str, expected: ScalarExpression| {
            let state = ParserState::new(input);

            let mut result = KqlParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test(
            "1",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );
        run_test(
            "(1)",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );
        run_test(
            "1e1",
            ScalarExpression::Static(StaticScalarExpression::Double(DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1e1,
            ))),
        );
        run_test(
            "real(1)",
            ScalarExpression::Static(StaticScalarExpression::Double(DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.0,
            ))),
        );
        run_test(
            "datetime(6/9/2025)",
            ScalarExpression::Static(StaticScalarExpression::DateTime(
                DateTimeScalarExpression::new(
                    QueryLocation::new_fake(),
                    create_utc(2025, 6, 9, 0, 0, 0, 0),
                ),
            )),
        );
        run_test(
            "true",
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
        );
        run_test(
            "false",
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            )),
        );
        run_test(
            "(true == true)",
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
        );
        run_test(
            "\"hello world\"",
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
        );
        run_test(
            "identifier",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                    StringScalarExpression::new(QueryLocation::new_fake(), "identifier"),
                )]),
            )),
        );
        run_test(
            "iff(true, 0, 1)",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
            )),
        );
    }
}
