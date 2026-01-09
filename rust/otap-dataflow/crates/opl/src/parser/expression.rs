// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    AndLogicalExpression, BinaryMathematicalScalarExpression, DoubleScalarExpression, DoubleValue,
    EqualToLogicalExpression, Expression, IntegerScalarExpression, IntegerValue, LogicalExpression,
    MathScalarExpression, NotLogicalExpression, NullScalarExpression, OrLogicalExpression,
    QueryLocation, ScalarExpression, SourceScalarExpression, StaticScalarExpression,
    StringScalarExpression, ValueAccessor,
};
use data_engine_parser_abstractions::{
    ParserError, parse_standard_bool_literal, parse_standard_double_literal,
    parse_standard_integer_literal, parse_standard_string_literal, to_query_location,
};
use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

fn parse_next_child_rule<F, E>(
    rules: &mut Pairs<'_, Rule>,
    expected_rule: Rule,
    child_parser: F,
) -> Result<E, ParserError>
where
    F: Fn(Pair<'_, Rule>) -> Result<E, ParserError>,
{
    if let Some(rule) = rules.next() {
        if rule.as_rule() == expected_rule {
            child_parser(rule)
        } else {
            todo!("invalid rule")
        }
    } else {
        todo!("no child rule")
    }
}

fn parse_maybe_right<F, E>(
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
        todo!("invalid right rule")
    }
}

fn parse_expected_right<F, E>(
    rules: &mut Pairs<'_, Rule>,
    expected_rule: Rule,
    parser: F,
) -> Result<E, ParserError>
where
    F: Fn(Pair<'_, Rule>) -> Result<E, ParserError>,
{
    let right_expr = parse_maybe_right(rules, expected_rule, parser).transpose()?;
    match right_expr {
        Some(r) => Ok(r),
        None => {
            todo!("Error no expected right")
        }
    }
}

pub(crate) fn parse_expression(rule: Pair<'_, Rule>) -> Result<LogicalExpression, ParserError> {
    println!("expression rule {:#?}", rule);
    let mut inner_rules = rule.into_inner();
    parse_next_child_rule(&mut inner_rules, Rule::or_expression, parse_or_expression)
}

pub(crate) fn parse_or_expression(rule: Pair<'_, Rule>) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr =
        parse_next_child_rule(&mut inner_rules, Rule::and_expression, parse_and_expression)?;
    let maybe_right = parse_maybe_right(&mut inner_rules, Rule::or_expression, parse_or_expression)
        .transpose()?;

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
        parse_next_child_rule(&mut inner_rules, Rule::rel_expression, parse_rel_expression)?.into();

    let maybe_right =
        parse_maybe_right(&mut inner_rules, Rule::and_expression, parse_and_expression)
            .transpose()?;

    Ok(match maybe_right {
        Some(right_expr) => LogicalExpression::And(AndLogicalExpression::new(
            query_location,
            left_expr,
            right_expr,
        )),
        None => left_expr,
    })
}

// TODO comment on what this is about
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
        Rule::additive_expression,
        parse_additive_expression,
    )?
    .into();

    if let Some(op_rule) = inner_rules.next() {
        let parsed_right =
            parse_expected_right(&mut inner_rules, Rule::rel_expression, parse_rel_expression)?;
        let right_expr = match parsed_right {
            LogicalOrScalarExpr::Scalar(s) => s,
            LogicalOrScalarExpr::Logical(l) => ScalarExpression::Logical(Box::new(l)),
        };
        let expr = match op_rule.as_rule() {
            Rule::rel_op_eq => LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                query_location,
                left_expr,
                right_expr,
                true,
            )),
            Rule::rel_op_neq => LogicalExpression::Not(NotLogicalExpression::new(
                query_location.clone(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    query_location,
                    left_expr,
                    right_expr,
                    true,
                )),
            )),
            _ => {
                todo!("invalid rel expr op {op_rule:?}")
            }
        };

        Ok(expr.into())
    } else {
        Ok(left_expr.into())
    }
}

pub(crate) fn parse_additive_expression(
    rule: Pair<'_, Rule>,
) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr = parse_next_child_rule(
        &mut inner_rules,
        Rule::multiplicative_expression,
        parse_multiplicative_expression,
    )?
    .into();

    if let Some(op_rule) = inner_rules.next() {
        let right_expr = parse_expected_right(
            &mut inner_rules,
            Rule::additive_expression,
            parse_additive_expression,
        )?
        .into();

        let math_expr =
            BinaryMathematicalScalarExpression::new(query_location.clone(), left_expr, right_expr);

        Ok(ScalarExpression::Math(match op_rule.as_rule() {
            Rule::additive_op_add => MathScalarExpression::Add(math_expr),
            Rule::additive_op_sub => MathScalarExpression::Subtract(math_expr),
            _ => {
                todo!("invalid add expr op")
            }
        })
        .into())
    } else {
        Ok(left_expr.into())
    }
}

pub(crate) fn parse_multiplicative_expression(
    rule: Pair<'_, Rule>,
) -> Result<LogicalOrScalarExpr, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr = parse_next_child_rule(
        &mut inner_rules,
        Rule::unary_expression,
        parse_unary_expression,
    )?
    .into();

    if let Some(op_rule) = inner_rules.next() {
        let right_expr = parse_expected_right(
            &mut inner_rules,
            Rule::multiplicative_expression,
            parse_multiplicative_expression,
        )?
        .into();

        let math_expr =
            BinaryMathematicalScalarExpression::new(query_location.clone(), left_expr, right_expr);

        Ok(ScalarExpression::Math(match op_rule.as_rule() {
            Rule::multiplicative_op_mul => MathScalarExpression::Multiply(math_expr),
            Rule::multiplicative_op_div => MathScalarExpression::Divide(math_expr),
            Rule::multiplicative_op_mod => MathScalarExpression::Modulus(math_expr),
            _ => {
                todo!("invalid add expr op")
            }
        })
        .into())
    } else {
        Ok(left_expr.into())
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
            _ => {
                todo!("invalid rule in no mod unary")
            }
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
            _ => {
                todo!("invalid rules for scalar")
            }
        }
    } else {
        todo!("invalid number of rules in unary expression")
    }
}

fn parse_number_literal(rule: Pair<'_, Rule>) -> Result<StaticScalarExpression, ParserError> {
    let mut inner_rules = rule.into_inner();
    if let Some(rule) = inner_rules.next() {
        match rule.as_rule() {
            Rule::integer_literal => parse_standard_integer_literal(rule),
            Rule::float_literal => parse_standard_double_literal(rule, None),
            _ => {
                todo!("invalid rule in number literal")
            }
        }
    } else {
        todo!("no rule inside number literal")
    }
}

fn negate_number_literal(
    static_scalar_expr: StaticScalarExpression,
) -> Result<StaticScalarExpression, ParserError> {
    Ok(match static_scalar_expr {
        StaticScalarExpression::Integer(int_expr) => {
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                int_expr.get_query_location().clone(),
                int_expr.get_value(),
            ))
        }
        StaticScalarExpression::Double(float_expr) => {
            StaticScalarExpression::Double(DoubleScalarExpression::new(
                float_expr.get_query_location().clone(),
                float_expr.get_value(),
            ))
        }
        _ => {
            todo!("invalid negated float")
        }
    })
}

fn parse_member_expression(rule: Pair<'_, Rule>) -> Result<LogicalOrScalarExpr, ParserError> {
    let mut inner_rules = rule.into_inner();
    if let Some(rule) = inner_rules.next() {
        match rule.as_rule() {
            Rule::primitive_expression => parse_primitive_expression(rule),
            _ => {
                todo!("invalid rule in member expression")
            }
        }
    } else {
        todo!("no inner rule in member expression")
    }
}

fn parse_primitive_expression(rule: Pair<'_, Rule>) -> Result<LogicalOrScalarExpr, ParserError> {
    println!("primitive rule {rule:#?}");
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    if let Some(rule) = inner_rules.next() {
        match rule.as_rule() {
            Rule::identifier_expression => {
                let query_location = to_query_location(&rule);
                let value_accessor =
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
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
            Rule::bool_true_token | Rule::bool_false_token => {
                Ok(ScalarExpression::Static(parse_standard_bool_literal(rule)).into())
            }
            Rule::null_token => Ok(ScalarExpression::Static(StaticScalarExpression::Null(
                NullScalarExpression::new(query_location),
            ))
            .into()),
            _ => {
                todo!("invalid token")
            }
        }
    } else {
        todo!("no inner rule in member expression")
    }
}

#[cfg(test)]
mod test {
    use data_engine_expressions::{
        LogicalExpression, QueryLocation, ScalarExpression, StaticScalarExpression,
        StringScalarExpression,
    };
    use pest::Parser;

    use crate::parser::{Rule, pest::OplPestParser};

    use super::parse_expression;

    #[test]
    fn test_parse_primitive() {
        let mut rules = OplPestParser::parse(Rule::expression, "\"hello\"").unwrap();
        assert_eq!(rules.len(), 1);
        let result = parse_expression(rules.next().unwrap()).unwrap();
        let expected =
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::String(
                StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
            )));
        assert_eq!(result, expected)
    }
}
