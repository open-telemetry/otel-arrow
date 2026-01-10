// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    AndLogicalExpression, BinaryMathematicalScalarExpression, BooleanScalarExpression,
    DoubleScalarExpression, DoubleValue, EqualToLogicalExpression, Expression,
    IntegerScalarExpression, IntegerValue, LogicalExpression, MathScalarExpression,
    NotLogicalExpression, NullScalarExpression, OrLogicalExpression, QueryLocation,
    ScalarExpression, SourceScalarExpression, StaticScalarExpression, StringScalarExpression,
    ValueAccessor,
};
use data_engine_parser_abstractions::{
    ParserError, parse_standard_double_literal, parse_standard_integer_literal,
    parse_standard_string_literal, to_query_location,
};
use pest::iterators::{Pair, Pairs};

use crate::parser::{Rule, invalid_child_rule_error};

fn parse_next_child_rule<F, E>(
    rules: &mut Pairs<'_, Rule>,
    expected_rule: Rule,
    child_parser: F,
) -> Option<Result<E, ParserError>>
where
    F: Fn(Pair<'_, Rule>) -> Result<E, ParserError>,
{
    rules.next().map(|rule| {
        if rule.as_rule() == expected_rule {
            child_parser(rule)
        } else {
            Err(ParserError::SyntaxError(
                to_query_location(&rule),
                format!("Invalid rule found {rule} expected {expected_rule:?}"),
            ))
        }
    })
}

fn parse_right<F, E>(
    rules: &mut Pairs<'_, Rule>,
    expected_rule: Rule,
    parser: F,
) -> Option<Result<E, ParserError>>
where
    F: Fn(Pair<'_, Rule>) -> Result<E, ParserError>,
{
    let right_rule = rules.next()?;
    if right_rule.as_rule() == expected_rule {
        Some(parser(right_rule))
    } else {
        Some(Err(ParserError::SyntaxError(
            to_query_location(&right_rule),
            format!("Invalid rule found {right_rule} expected {expected_rule:?}"),
        )))
    }
}

pub(crate) fn parse_expression(rule: Pair<'_, Rule>) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    parse_next_child_rule(&mut inner_rules, Rule::or_expression, parse_or_expression)
        .transpose()?
        .ok_or_else(|| no_inner_rule_error(query_location))
}

fn no_inner_rule_error(query_location: QueryLocation) -> ParserError {
    ParserError::SyntaxError(
        query_location,
        "No inner rule found in expression".to_string(),
    )
}

pub(crate) fn parse_or_expression(rule: Pair<'_, Rule>) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr =
        parse_next_child_rule(&mut inner_rules, Rule::and_expression, parse_and_expression)
            .transpose()?
            .ok_or_else(|| no_inner_rule_error(query_location.clone()))?;

    let maybe_right =
        parse_right(&mut inner_rules, Rule::or_expression, parse_or_expression).transpose()?;

    Ok(match maybe_right {
        Some(right_expr) => LogicalExpression::Or(OrLogicalExpression::new(
            query_location,
            left_expr,
            right_expr,
        )),
        None => left_expr,
    })
}

pub(crate) fn parse_and_expression(rule: Pair<'_, Rule>) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr =
        parse_next_child_rule(&mut inner_rules, Rule::rel_expression, parse_rel_expression)
            .transpose()?
            .ok_or_else(|| no_inner_rule_error(query_location.clone()))?
            .into();

    let maybe_right =
        parse_right(&mut inner_rules, Rule::and_expression, parse_and_expression).transpose()?;

    Ok(match maybe_right {
        Some(right_expr) => LogicalExpression::And(AndLogicalExpression::new(
            query_location,
            left_expr,
            right_expr,
        )),
        None => left_expr,
    })
}

#[derive(Debug, PartialEq)]
pub(crate) enum LogicalOrScalarExpr {
    Logical(LogicalExpression),
    Scalar(ScalarExpression),
}

impl From<LogicalExpression> for LogicalOrScalarExpr {
    fn from(expr: LogicalExpression) -> Self {
        Self::Logical(expr)
    }
}

impl From<ScalarExpression> for LogicalOrScalarExpr {
    fn from(expr: ScalarExpression) -> Self {
        Self::Scalar(expr)
    }
}

impl From<LogicalOrScalarExpr> for LogicalExpression {
    fn from(value: LogicalOrScalarExpr) -> Self {
        match value {
            LogicalOrScalarExpr::Logical(l) => l,
            LogicalOrScalarExpr::Scalar(s) => LogicalExpression::Scalar(s),
        }
    }
}

impl From<LogicalOrScalarExpr> for ScalarExpression {
    fn from(value: LogicalOrScalarExpr) -> Self {
        match value {
            LogicalOrScalarExpr::Logical(l) => Self::Logical(Box::new(l)),
            LogicalOrScalarExpr::Scalar(s) => s,
        }
    }
}

pub(crate) fn parse_rel_expression(
    rule: Pair<'_, Rule>,
) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr = parse_next_child_rule(
        &mut inner_rules,
        Rule::multiplicative_expression,
        parse_multiplicative_expression,
    )
    .transpose()?
    .ok_or_else(|| no_inner_rule_error(query_location.clone()))?;

    if let Some(op_rule) = inner_rules.next() {
        let right_expr = parse_right(&mut inner_rules, Rule::rel_expression, parse_rel_expression)
            .transpose()?
            .ok_or_else(|| {
                ParserError::SyntaxError(
                    query_location.clone(),
                    "Expected right expression in relational expression".to_string(),
                )
            })?
            .into();

        let expr = match op_rule.as_rule() {
            Rule::rel_op_eq => LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                query_location,
                left_expr.into(),
                right_expr,
                true,
            )),
            Rule::rel_op_neq => LogicalExpression::Not(NotLogicalExpression::new(
                query_location.clone(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    query_location,
                    left_expr.into(),
                    right_expr,
                    true,
                )),
            )),
            invalid_rule => {
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::rel_expression,
                    invalid_rule,
                ));
            }
        };

        Ok(expr.into())
    } else {
        Ok(left_expr)
    }
}

pub(crate) fn parse_multiplicative_expression(
    rule: Pair<'_, Rule>,
) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr = parse_next_child_rule(
        &mut inner_rules,
        Rule::additive_expression,
        parse_additive_expression,
    )
    .transpose()?
    .ok_or_else(|| no_inner_rule_error(query_location.clone()))?;

    if let Some(op_rule) = inner_rules.next() {
        let right_expr = parse_right(
            &mut inner_rules,
            Rule::multiplicative_expression,
            parse_multiplicative_expression,
        )
        .transpose()?
        .ok_or_else(|| {
            ParserError::SyntaxError(
                query_location.clone(),
                "Expected right expression in multiplicative expression".to_string(),
            )
        })?
        .into();

        let math_expr = BinaryMathematicalScalarExpression::new(
            query_location.clone(),
            left_expr.into(),
            right_expr,
        );

        Ok(ScalarExpression::Math(match op_rule.as_rule() {
            Rule::multiplicative_op_mul => MathScalarExpression::Multiply(math_expr),
            Rule::multiplicative_op_div => MathScalarExpression::Divide(math_expr),
            Rule::multiplicative_op_mod => MathScalarExpression::Modulus(math_expr),
            invalid_rule => {
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::multiplicative_expression,
                    invalid_rule,
                ));
            }
        })
        .into())
    } else {
        Ok(left_expr)
    }
}

pub(crate) fn parse_additive_expression(
    rule: Pair<'_, Rule>,
) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr = parse_next_child_rule(
        &mut inner_rules,
        Rule::unary_expression,
        parse_unary_expression,
    )
    .transpose()?
    .ok_or_else(|| no_inner_rule_error(query_location.clone()))?;

    if let Some(op_rule) = inner_rules.next() {
        let right_expr = parse_right(
            &mut inner_rules,
            Rule::additive_expression,
            parse_additive_expression,
        )
        .transpose()?
        .ok_or_else(|| {
            ParserError::SyntaxError(
                query_location.clone(),
                "Expected right expression in additive expression".to_string(),
            )
        })?
        .into();

        let math_expr = BinaryMathematicalScalarExpression::new(
            query_location.clone(),
            left_expr.into(),
            right_expr,
        );

        Ok(ScalarExpression::Math(match op_rule.as_rule() {
            Rule::additive_op_add => MathScalarExpression::Add(math_expr),
            Rule::additive_op_sub => MathScalarExpression::Subtract(math_expr),
            invalid_rule => {
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::additive_expression,
                    invalid_rule,
                ));
            }
        })
        .into())
    } else {
        Ok(left_expr)
    }
}

fn parse_unary_expression(rule: Pair<'_, Rule>) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = QueryLocation::new_fake();
    let mut inner_rules = rule.into_inner();
    if inner_rules.len() == 1 {
        // safety: we've checked the length of the iterator above
        let rule = inner_rules.next().expect("one rule");
        match rule.as_rule() {
            Rule::number_literal => {
                Ok(ScalarExpression::Static(parse_number_literal(rule)?).into())
            }
            Rule::member_expression => parse_member_expression(rule),
            invalid_rule => Err(invalid_child_rule_error(
                query_location,
                Rule::unary_expression,
                invalid_rule,
            )),
        }
    } else if inner_rules.len() == 2 {
        // safety: we've checked that the Pairs iter has len 2, so w can call next.expect twice
        let modifier_rule = inner_rules.next().expect("two rules");
        let value_rule = inner_rules.next().expect("two rules");

        match (modifier_rule.as_rule(), value_rule.as_rule()) {
            (Rule::negate_token, Rule::number_literal) => {
                let number_expr = parse_number_literal(value_rule)?;
                let expr = ScalarExpression::Static(negate_number_literal(number_expr)?);
                Ok(expr.into())
            }
            (Rule::not_token, Rule::unary_expression) => {
                let value_expr = parse_unary_expression(value_rule)?;
                Ok(LogicalExpression::Not(NotLogicalExpression::new(
                    query_location,
                    value_expr.into(),
                ))
                .into())
            }
            invalid_rules => Err(ParserError::SyntaxError(
                query_location,
                format!("Invalid unary expression with modifier {invalid_rules:?}"),
            )),
        }
    } else {
        Err(ParserError::SyntaxError(
            query_location,
            "Invalid number of rules in unary expression".to_string(),
        ))
    }
}

fn parse_number_literal(rule: Pair<'_, Rule>) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let rule = inner_rules
        .next()
        .ok_or_else(|| no_inner_rule_error(query_location.clone()))?;

    match rule.as_rule() {
        Rule::integer_literal => parse_standard_integer_literal(rule),
        Rule::float_literal => parse_standard_double_literal(rule, None),
        invalid_rule => Err(invalid_child_rule_error(
            query_location,
            Rule::number_literal,
            invalid_rule,
        )),
    }
}

fn negate_number_literal(
    static_scalar_expr: StaticScalarExpression,
) -> Result<StaticScalarExpression, ParserError> {
    Ok(match static_scalar_expr {
        StaticScalarExpression::Integer(int_expr) => {
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                int_expr.get_query_location().clone(),
                -int_expr.get_value(),
            ))
        }
        StaticScalarExpression::Double(float_expr) => {
            StaticScalarExpression::Double(DoubleScalarExpression::new(
                float_expr.get_query_location().clone(),
                -float_expr.get_value(),
            ))
        }
        invalid_rule => {
            return Err(ParserError::SyntaxError(
                invalid_rule.get_query_location().clone(),
                format!("Invalid static scalar expression to negate: {invalid_rule:?}"),
            ));
        }
    })
}

fn parse_member_expression(rule: Pair<'_, Rule>) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let rule = inner_rules
        .next()
        .ok_or_else(|| no_inner_rule_error(query_location.clone()))?;

    match rule.as_rule() {
        Rule::index_expression => parse_index_expression(rule),
        Rule::primitive_expression => parse_primitive_expression(rule),
        Rule::attribute_selection_expression => parse_attribute_selection_expression(rule),
        invalid_rule => Err(invalid_child_rule_error(
            query_location,
            Rule::member_expression,
            invalid_rule,
        )),
    }
}

fn parse_index_expression(rule: Pair<'_, Rule>) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    if inner_rules.len() == 2 {
        // Safety: we've checked the length of the iterator
        let source_rule = inner_rules.next().expect("two rules");
        let index_rule = inner_rules.next().expect("two rules");

        match (source_rule.as_rule(), index_rule.as_rule()) {
            (Rule::identifier_expression, Rule::member_expression) => {
                let source_expr = ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(query_location.clone(), source_rule.as_str()),
                ));
                let index_expr = parse_member_expression(index_rule)?;

                let value_accessor =
                    ValueAccessor::new_with_selectors(vec![source_expr, index_expr.into()]);

                Ok(ScalarExpression::Source(SourceScalarExpression::new(
                    query_location,
                    value_accessor,
                ))
                .into())
            }
            invalid_rules => Err(ParserError::SyntaxError(
                query_location,
                format!("Invalid unary expression with modifier {invalid_rules:?}"),
            )),
        }
    } else {
        Err(ParserError::SyntaxError(
            query_location,
            "Invalid number of rules in index expression".to_string(),
        ))
    }
}

fn parse_attribute_selection_expression(
    rule: Pair<'_, Rule>,
) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let source_rule = inner_rules
        .next()
        .ok_or_else(|| no_inner_rule_error(query_location.clone()))?;

    let mut selections = vec![ScalarExpression::Static(StaticScalarExpression::String(
        StringScalarExpression::new(query_location.clone(), source_rule.as_str()),
    ))];

    for rule in inner_rules {
        match rule.as_rule() {
            Rule::identifier_expression => {
                let selection_expr = ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(query_location.clone(), rule.as_str()),
                ));
                selections.push(selection_expr);
            }
            invalid_rule => {
                return Err(invalid_child_rule_error(
                    query_location.clone(),
                    Rule::attribute_selection_expression,
                    invalid_rule,
                ));
            }
        }
    }

    let value_accessor = ValueAccessor::new_with_selectors(selections);
    Ok(
        ScalarExpression::Source(SourceScalarExpression::new(query_location, value_accessor))
            .into(),
    )
}

fn parse_primitive_expression(rule: Pair<'_, Rule>) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let rule = inner_rules
        .next()
        .ok_or_else(|| no_inner_rule_error(query_location.clone()))?;

    match rule.as_rule() {
        Rule::identifier_expression => {
            let query_location = to_query_location(&rule);
            let value_accessor = ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    query_location.clone(),
                    rule.as_str(),
                )),
            )]);

            Ok(ScalarExpression::Source(SourceScalarExpression::new(
                query_location,
                value_accessor,
            ))
            .into())
        }
        Rule::string_literal => {
            Ok(ScalarExpression::Static(parse_standard_string_literal(rule)).into())
        }
        Rule::bool_true_token => Ok(ScalarExpression::Static(StaticScalarExpression::Boolean(
            BooleanScalarExpression::new(query_location, true),
        ))
        .into()),
        Rule::bool_false_token => Ok(ScalarExpression::Static(StaticScalarExpression::Boolean(
            BooleanScalarExpression::new(query_location, false),
        ))
        .into()),

        Rule::null_token => Ok(ScalarExpression::Static(StaticScalarExpression::Null(
            NullScalarExpression::new(query_location),
        ))
        .into()),
        Rule::expression => parse_expression(rule).map(|le| le.into()),
        invalid_rule => Err(invalid_child_rule_error(
            query_location,
            Rule::primitive_expression,
            invalid_rule,
        )),
    }
}
#[cfg(test)]
mod test {
    use data_engine_expressions::{
        AndLogicalExpression, BinaryMathematicalScalarExpression, BooleanScalarExpression,
        DoubleScalarExpression, EqualToLogicalExpression, IntegerScalarExpression,
        LogicalExpression, MathScalarExpression, NotLogicalExpression, NullScalarExpression,
        OrLogicalExpression, QueryLocation, ScalarExpression, SourceScalarExpression,
        StaticScalarExpression, StringScalarExpression, ValueAccessor,
    };
    use pest::Parser;
    use pretty_assertions::assert_eq;

    use crate::parser::{
        Rule,
        expression::{
            LogicalOrScalarExpr, parse_additive_expression, parse_attribute_selection_expression,
            parse_expression, parse_multiplicative_expression, parse_rel_expression,
            parse_unary_expression,
        },
        pest::OplPestParser,
    };

    #[test]
    fn test_parse_unary_expression_static_primitives() {
        let test_cases = [
            (
                "\"hello\"",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello",
                )),
            ),
            (
                "123",
                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    123,
                )),
            ),
            (
                "-456",
                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    -456,
                )),
            ),
            (
                "1.23",
                StaticScalarExpression::Double(DoubleScalarExpression::new(
                    QueryLocation::new_fake(),
                    1.23,
                )),
            ),
            (
                "-4.56",
                StaticScalarExpression::Double(DoubleScalarExpression::new(
                    QueryLocation::new_fake(),
                    -4.56,
                )),
            ),
            (
                "true",
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    QueryLocation::new_fake(),
                    true,
                )),
            ),
            (
                "false",
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    QueryLocation::new_fake(),
                    false,
                )),
            ),
            (
                "null",
                StaticScalarExpression::Null(NullScalarExpression::new(QueryLocation::new_fake())),
            ),
        ];

        for (input, expected) in test_cases {
            let mut rules = OplPestParser::parse(Rule::unary_expression, input).unwrap();
            assert_eq!(rules.len(), 1);
            let result: ScalarExpression = parse_unary_expression(rules.next().unwrap())
                .unwrap()
                .into();
            let expected = ScalarExpression::Static(expected.clone());
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_unary_not_bool() {
        let input = "not false";

        let mut rules = OplPestParser::parse(Rule::unary_expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let result: LogicalExpression = parse_unary_expression(rules.next().unwrap())
            .unwrap()
            .into();
        let expected = LogicalExpression::Not(NotLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            ))),
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_unary_source_identifier() {
        let input = "some_field";
        let mut rules = OplPestParser::parse(Rule::unary_expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let result: ScalarExpression = parse_unary_expression(rules.next().unwrap())
            .unwrap()
            .into();

        let expected = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "some_field",
                )),
            )]),
        ));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_unary_source_index() {
        let input = "attributes[\"key\"]";
        let mut rules = OplPestParser::parse(Rule::unary_expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let result: ScalarExpression = parse_unary_expression(rules.next().unwrap())
            .unwrap()
            .into();

        let expected = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "key"),
                )),
            ]),
        ));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_attribute_selection_expression() {
        let input = "attributes.key.subkey";
        let mut rules = OplPestParser::parse(Rule::attribute_selection_expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let result: ScalarExpression = parse_attribute_selection_expression(rules.next().unwrap())
            .unwrap()
            .into();

        let expected = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "key"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "subkey"),
                )),
            ]),
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_rel_expression_equal() {
        let input = "a == 10";
        let mut rules = OplPestParser::parse(Rule::expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let result = parse_expression(rules.next().unwrap()).unwrap();

        let expected = LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "a",
                    )),
                )]),
            )),
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
            )),
            true,
        ));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_additive_expressions() {
        let test_cases = vec![
            (
                "1 + 5",
                MathScalarExpression::Add(BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                    )),
                )),
            ),
            (
                "2 - 6",
                MathScalarExpression::Subtract(BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 6),
                    )),
                )),
            ),
            (
                "1 + 2 - 3",
                MathScalarExpression::Add(BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Math(MathScalarExpression::Subtract(
                        BinaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                        ),
                    )),
                )),
            ),
        ];

        for (input, math_expr) in test_cases {
            let mut rules = OplPestParser::parse(Rule::additive_expression, input).unwrap();
            assert_eq!(rules.len(), 1);
            let result: LogicalOrScalarExpr =
                parse_additive_expression(rules.next().unwrap()).unwrap();
            let expected: LogicalOrScalarExpr = ScalarExpression::Math(math_expr).into();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_multiplicative_expressions() {
        let test_cases = vec![
            (
                "4 * 2",
                MathScalarExpression::Multiply(BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                    )),
                )),
            ),
            (
                "8 / 4",
                MathScalarExpression::Divide(BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 8),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                )),
            ),
            (
                "9 % 2",
                MathScalarExpression::Modulus(BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 9),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                    )),
                )),
            ),
            (
                "4 * 2 / 8",
                MathScalarExpression::Multiply(BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                    ScalarExpression::Math(MathScalarExpression::Divide(
                        BinaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 8),
                            )),
                        ),
                    )),
                )),
            ),
        ];

        for (input, math_expr) in test_cases {
            let mut rules = OplPestParser::parse(Rule::multiplicative_expression, input).unwrap();
            assert_eq!(rules.len(), 1);
            let result: LogicalOrScalarExpr =
                parse_multiplicative_expression(rules.next().unwrap()).unwrap();
            let expected: LogicalOrScalarExpr = ScalarExpression::Math(math_expr).into();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parser_math_order_precedent() {
        let input = "1 + 2 * 3 - 4 / 5 % 6";
        let mut rules = OplPestParser::parse(Rule::rel_expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let result: ScalarExpression = parse_rel_expression(rules.next().unwrap()).unwrap().into();

        let one_plus_two = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                )),
            ),
        ));

        let three_minus_four = ScalarExpression::Math(MathScalarExpression::Subtract(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                )),
            ),
        ));

        let five_mod_six = ScalarExpression::Math(MathScalarExpression::Modulus(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 6),
                )),
            ),
        ));

        let expected = ScalarExpression::Math(MathScalarExpression::Multiply(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                one_plus_two,
                ScalarExpression::Math(MathScalarExpression::Divide(
                    BinaryMathematicalScalarExpression::new(
                        QueryLocation::new_fake(),
                        three_minus_four,
                        five_mod_six,
                    ),
                )),
            ),
        ));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_req_expression_not_equal() {
        let input = "b != 20";
        let mut rules = OplPestParser::parse(Rule::expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let result = parse_expression(rules.next().unwrap()).unwrap();

        let expected = LogicalExpression::Not(NotLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "b",
                        )),
                    )]),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 20),
                )),
                true,
            )),
        ));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_expression_simple_logical_with_precedence() {
        let input = "a == 10 or b == 20 and c == 30";
        let mut rules = OplPestParser::parse(Rule::expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let result = parse_expression(rules.next().unwrap()).unwrap();

        let expected = LogicalExpression::Or(OrLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "a",
                        )),
                    )]),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                )),
                true,
            )),
            LogicalExpression::And(AndLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "b",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 20),
                    )),
                    true,
                )),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "c",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 30),
                    )),
                    true,
                )),
            )),
        ));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_logical_expression_simple() {
        let input = "(x or y) and z";
        let mut rules = OplPestParser::parse(Rule::expression, input).unwrap();
        assert_eq!(rules.len(), 1);

        fn source_logical_expr(field_name: &str) -> LogicalExpression {
            LogicalExpression::Scalar(ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        field_name,
                    )),
                )]),
            )))
        }

        let result = parse_expression(rules.next().unwrap()).unwrap();
        let expected = LogicalExpression::And(AndLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Or(OrLogicalExpression::new(
                QueryLocation::new_fake(),
                source_logical_expr("x"),
                source_logical_expr("y"),
            )),
            source_logical_expr("z"),
        ));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_logical_expression_with_not() {
        let input = "not (x or y) and not z or w";
        let mut rules = OplPestParser::parse(Rule::expression, input).unwrap();
        assert_eq!(rules.len(), 1);

        fn source_logical_expr(field_name: &str) -> LogicalExpression {
            LogicalExpression::Scalar(ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        field_name,
                    )),
                )]),
            )))
        }

        let result = parse_expression(rules.next().unwrap()).unwrap();
        let expected = LogicalExpression::Or(OrLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::And(AndLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Not(NotLogicalExpression::new(
                    QueryLocation::new_fake(),
                    LogicalExpression::Or(OrLogicalExpression::new(
                        QueryLocation::new_fake(),
                        source_logical_expr("x"),
                        source_logical_expr("y"),
                    )),
                )),
                LogicalExpression::Not(NotLogicalExpression::new(
                    QueryLocation::new_fake(),
                    source_logical_expr("z"),
                )),
            )),
            source_logical_expr("w"),
        ));

        assert_eq!(result, expected);
    }
}
