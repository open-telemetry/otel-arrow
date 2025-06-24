use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule, logical_expressions::parse_logical_expression, scalar_function_expressions::*,
    scalar_primitive_expressions::*,
};

pub(crate) fn parse_scalar_expression(
    scalar_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let scalar_rule = scalar_expression_rule.into_inner().next().unwrap();

    match scalar_rule.as_rule() {
        Rule::real_expression => Ok(ScalarExpression::Static(parse_real_expression(
            scalar_rule,
        )?)),
        Rule::datetime_expression => Ok(ScalarExpression::Static(parse_datetime_expression(
            scalar_rule,
        )?)),
        Rule::logical_expression => Ok(ScalarExpression::Logical(
            parse_logical_expression(scalar_rule, state)?.into(),
        )),
        Rule::true_literal | Rule::false_literal => Ok(ScalarExpression::Static(
            parse_standard_bool_literal(scalar_rule),
        )),
        Rule::double_literal => Ok(ScalarExpression::Static(parse_double_literal(scalar_rule)?)),
        Rule::integer_literal => Ok(ScalarExpression::Static(parse_integer_literal(
            scalar_rule,
        )?)),
        Rule::string_literal => Ok(ScalarExpression::Static(parse_string_literal(scalar_rule))),
        Rule::accessor_expression => parse_accessor_expression(scalar_rule, state),
        Rule::conditional_expression => parse_conditional_expression(scalar_rule, state),
        Rule::scalar_expression => parse_scalar_expression(scalar_rule, state),
        _ => panic!("Unexpected rule in scalar_expression: {}", scalar_rule),
    }
}

#[cfg(test)]
mod tests {
    use crate::KqlParser;

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
                "iif(true, 0, 1)"
            ],
            &["!"],
        );
    }
}
