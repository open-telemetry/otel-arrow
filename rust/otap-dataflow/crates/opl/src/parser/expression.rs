// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    AndLogicalExpression, BinaryMathematicalScalarExpression, EqualToLogicalExpression,
    IntegerScalarExpression, LogicalExpression, MathScalarExpression, NotLogicalExpression,
    OrLogicalExpression, QueryLocation, ScalarExpression, StaticScalarExpression,
    StringScalarExpression,
};
use data_engine_parser_abstractions::{
    ParserError, parse_standard_double_literal, parse_standard_integer_literal, to_query_location,
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
    let parsed_left =
        parse_next_child_rule(&mut inner_rules, Rule::rel_expression, parse_rel_expression)?;

    let left_expr = match parsed_left {
        ParsedRelExpression::Logical(l) => l,
        ParsedRelExpression::Scalar(s) => LogicalExpression::Scalar(s),
    };
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

pub(crate) enum ParsedRelExpression {
    Logical(LogicalExpression),
    Scalar(ScalarExpression),
}

pub(crate) fn parse_rel_expression(
    rule: Pair<'_, Rule>,
) -> Result<ParsedRelExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr = parse_next_child_rule(
        &mut inner_rules,
        Rule::additive_expression,
        parse_additive_expression,
    )?;

    if let Some(op_rule) = inner_rules.next() {
        let parsed_right =
            parse_expected_right(&mut inner_rules, Rule::rel_expression, parse_rel_expression)?;
        let right_expr = match parsed_right {
            ParsedRelExpression::Scalar(s) => s,
            ParsedRelExpression::Logical(l) => ScalarExpression::Logical(Box::new(l)),
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
                todo!("invalid rel expr op")
            }
        };

        Ok(ParsedRelExpression::Logical(expr))
    } else {
        Ok(ParsedRelExpression::Scalar(left_expr))
    }
}

pub(crate) fn parse_additive_expression(
    rule: Pair<'_, Rule>,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr = parse_next_child_rule(
        &mut inner_rules,
        Rule::multiplicative_expression,
        parse_multiplicative_expression,
    )?;

    if let Some(op_rule) = inner_rules.next() {
        let right_expr = parse_expected_right(
            &mut inner_rules,
            Rule::additive_expression,
            parse_additive_expression,
        )?;

        let math_expr =
            BinaryMathematicalScalarExpression::new(query_location.clone(), left_expr, right_expr);

        Ok(ScalarExpression::Math(match op_rule.as_rule() {
            Rule::additive_op_add => MathScalarExpression::Add(math_expr),
            Rule::additive_op_sub => MathScalarExpression::Subtract(math_expr),
            _ => {
                todo!("invalid add expr op")
            }
        }))
    } else {
        Ok(left_expr)
    }
}

pub(crate) fn parse_multiplicative_expression(
    rule: Pair<'_, Rule>,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr = parse_next_child_rule(
        &mut inner_rules,
        Rule::unary_expression,
        parse_unary_expression,
    )?;

    if let Some(op_rule) = inner_rules.next() {
        let right_expr = parse_expected_right(
            &mut inner_rules,
            Rule::multiplicative_expression,
            parse_multiplicative_expression,
        )?;

        let math_expr =
            BinaryMathematicalScalarExpression::new(query_location.clone(), left_expr, right_expr);

        Ok(ScalarExpression::Math(match op_rule.as_rule() {
            Rule::multiplicative_op_mul => MathScalarExpression::Multiply(math_expr),
            Rule::multiplicative_op_div => MathScalarExpression::Divide(math_expr),
            Rule::multiplicative_op_mod => MathScalarExpression::Modulus(math_expr),
            _ => {
                todo!("invalid add expr op")
            }
        }))
    } else {
        Ok(left_expr)
    }
}

fn parse_unary_expression(rule: Pair<'_, Rule>) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();

    if inner_rules.len() == 1 {
        // safety: we've checked the length of the iterator above
        let rule = inner_rules.next().expect("one rule");
        match rule.as_rule() {
            Rule::number_literal => Ok(ScalarExpression::Static(parse_number_literal(rule)?)),
            Rule::member_expression => parse_member_expression(rule),
            _ => {
                todo!("invalid rule in no mod unary")
            }
        }
    } else {
        todo!()
    }
}

fn parse_number_literal(rule: Pair<'_, Rule>) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&rule);
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
    scalar_expr: StaticScalarExpression,
) -> Result<StaticScalarExpression, ParserError> {
    todo!()
}

fn parse_member_expression(rule: Pair<'_, Rule>) -> Result<ScalarExpression, ParserError> {
    todo!()
}
