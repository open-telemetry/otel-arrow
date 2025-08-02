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

pub(crate) fn parse_case_expression(
    case_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&case_expression_rule);

    let case_rules = case_expression_rule.into_inner();
    let mut conditions = Vec::new();
    let mut expressions = Vec::new();

    // The grammar ensures we have: logical_expression, scalar_expression, (logical_expression, scalar_expression)*, scalar_expression
    // So we alternate between conditions and expressions, with the last one being the else expression

    let rules_vec: Vec<_> = case_rules.collect();

    // We need at least 3 elements: condition, then_expr, else_expr
    if rules_vec.len() < 3 {
        return Err(ParserError::SyntaxError(
            query_location,
            "Case statement requires at least one condition, one then expression, and one else expression".to_string(),
        ));
    }

    // Process pairs of (condition, expression) until we reach the last element (else)
    let mut i = 0;
    while i < rules_vec.len() - 1 {
        // Parse condition
        let condition = parse_logical_expression(rules_vec[i].clone(), state)?;
        conditions.push(condition);
        i += 1;

        // Parse corresponding expression
        if i < rules_vec.len() - 1 {
            // Not the last element (which is else)
            let expression = parse_scalar_expression(rules_vec[i].clone(), state)?;
            expressions.push(expression);
            i += 1;
        } else {
            return Err(ParserError::SyntaxError(
                query_location,
                "Missing expression after condition in case statement".to_string(),
            ));
        }
    }

    // Parse the else expression (last element)
    let else_expression = parse_scalar_expression(rules_vec[rules_vec.len() - 1].clone(), state)?;

    Ok(ScalarExpression::Case(CaseScalarExpression::new(
        query_location,
        conditions,
        expressions,
        else_expression,
    )))
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_parse_conditional_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::conditional_expression, input).unwrap();

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

    #[test]
    fn test_parse_case_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::case_expression, input).unwrap();

            let expression = parse_case_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        // Test simple case with one condition
        run_test_success(
            "case(true, 1, 0)",
            ScalarExpression::Case(CaseScalarExpression::new(
                QueryLocation::new_fake(),
                vec![LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                ))],
                vec![ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))],
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            )),
        );

        // Test case with multiple conditions
        run_test_success(
            "case(1 > 0, \"positive\", false, \"negative\", \"zero\")",
            ScalarExpression::Case(CaseScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )),
                    )),
                    LogicalExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            false,
                        )),
                    )),
                ],
                vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "positive"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "negative"),
                    )),
                ],
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "zero"),
                )),
            )),
        );

        // Test case with complex OR conditions
        let run_test_success_with_state = |input: &str, expected: ScalarExpression| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );
            state.push_variable_name("key");

            let mut result = KqlPestParser::parse(Rule::case_expression, input).unwrap();

            let expression = parse_case_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        // Create first condition: key == 'foo' or key == 'FOO'
        let mut first_condition = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "key"),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "foo"),
                )),
                false,
            )),
        );
        first_condition.push_or(LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "key"),
                ValueAccessor::new(),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "FOO",
            ))),
            false,
        )));

        // Create second condition: key == 'bar' or key == 'BAR'
        let mut second_condition = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "key"),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "bar"),
                )),
                false,
            )),
        );
        second_condition.push_or(LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "key"),
                ValueAccessor::new(),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "BAR",
            ))),
            false,
        )));

        run_test_success_with_state(
            "case(key == 'foo' or key == 'FOO', 1, key == 'bar' or key == 'BAR', 2, 3)",
            ScalarExpression::Case(CaseScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    LogicalExpression::Chain(first_condition),
                    LogicalExpression::Chain(second_condition),
                ],
                vec![
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                    )),
                ],
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                )),
            )),
        );
    }
}
